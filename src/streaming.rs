//! Streaming SSE: converte il flusso OpenAI `chat.completion.chunk` in eventi
//! Anthropic Messages API, come fa l'adapter streaming di CCR/ai-gateway.
//!
//! OpenAI in ingresso (per chunk):
//!   data: {"choices":[{"delta":{"content":"..."},"finish_reason":null}]}
//!   data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"...","function":{"name":"...","arguments":"..."}}]},"finish_reason":null}]}
//!   data: [DONE]
//!
//! Anthropic in uscita:
//!   event: message_start
//!   event: content_block_start (text | tool_use | thinking)
//!   event: content_block_delta  (text_delta | input_json_delta | thinking_delta)
//!   event: content_block_stop
//!   event: message_delta  (stop_reason)
//!   event: message_stop

use bytes::Bytes;
use futures_util::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Stream che converte SSE OpenAI → SSE Anthropic chunk per chunk.
pub struct AnthropicSseStream<S> {
    inner: S,
    block_index: usize,
    current_block: Option<String>, // "text" | "thinking" | "tool"
    started: bool,
    finished: bool,
    output_tokens: u64,
    input_tokens: u64,
    model: String,
    message_id: String,
}

impl<S> AnthropicSseStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    pub fn new(inner: S, model: String, message_id: String, input_tokens: u64) -> Self {
        Self {
            inner,
            block_index: 0,
            current_block: None,
            started: false,
            finished: false,
            output_tokens: 0,
            input_tokens,
            model,
            message_id,
        }
    }

    fn start_event(&self) -> String {
        format!(
            "event: message_start\ndata: {}\n\n",
            serde_json::json!({
                "type": "message_start",
                "message": {
                    "id": self.message_id,
                    "type": "message",
                    "role": "assistant",
                    "content": [],
                    "model": self.model,
                    "stop_reason": null,
                    "stop_sequence": null,
                    "usage": { "input_tokens": self.input_tokens, "output_tokens": 0 }
                }
            })
        )
    }

    fn block_start_text(&self, index: usize) -> String {
        format!(
            "event: content_block_start\ndata: {}\n\n",
            serde_json::json!({
                "type": "content_block_start",
                "index": index,
                "content_block": { "type": "text", "text": "" }
            })
        )
    }

    fn block_start_tool(&self, index: usize, id: &str, name: &str) -> String {
        format!(
            "event: content_block_start\ndata: {}\n\n",
            serde_json::json!({
                "type": "content_block_start",
                "index": index,
                "content_block": { "type": "tool_use", "id": id, "name": name, "input": {} }
            })
        )
    }

    fn block_start_thinking(&self, index: usize) -> String {
        format!(
            "event: content_block_start\ndata: {}\n\n",
            serde_json::json!({
                "type": "content_block_start",
                "index": index,
                "content_block": { "type": "thinking", "thinking": "" }
            })
        )
    }

    fn text_delta(&self, index: usize, text: &str) -> String {
        format!(
            "event: content_block_delta\ndata: {}\n\n",
            serde_json::json!({
                "type": "content_block_delta",
                "index": index,
                "delta": { "type": "text_delta", "text": text }
            })
        )
    }

    fn thinking_delta(&self, index: usize, text: &str) -> String {
        format!(
            "event: content_block_delta\ndata: {}\n\n",
            serde_json::json!({
                "type": "content_block_delta",
                "index": index,
                "delta": { "type": "thinking_delta", "thinking": text }
            })
        )
    }

    fn tool_delta(&self, index: usize, partial_json: &str) -> String {
        format!(
            "event: content_block_delta\ndata: {}\n\n",
            serde_json::json!({
                "type": "content_block_delta",
                "index": index,
                "delta": { "type": "input_json_delta", "partial_json": partial_json }
            })
        )
    }

    fn block_stop(&self, index: usize) -> String {
        format!(
            "event: content_block_stop\ndata: {}\n\n",
            serde_json::json!({ "type": "content_block_stop", "index": index })
        )
    }

    fn message_delta(&self, stop_reason: &str) -> String {
        format!(
            "event: message_delta\ndata: {}\n\n",
            serde_json::json!({
                "type": "message_delta",
                "delta": { "stop_reason": stop_reason, "stop_sequence": null },
                "usage": { "output_tokens": self.output_tokens }
            })
        )
    }

    fn message_stop(&self) -> String {
        "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string()
    }

    fn ping(&self) -> String {
        "event: ping\ndata: {\"type\": \"ping\"}\n\n".to_string()
    }

    fn close_block(&mut self, out: &mut String) {
        if self.current_block.is_some() {
            out.push_str(&self.block_stop(self.block_index.saturating_sub(1)));
            self.current_block = None;
        }
    }
}

impl<S> Stream for AnthropicSseStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.started {
            self.started = true;
            let mut out = self.start_event();
            out.push_str(&self.ping());
            return Poll::Ready(Some(Ok(Bytes::from(out))));
        }

        if self.finished {
            return Poll::Ready(None);
        }

        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                let text = String::from_utf8_lossy(&chunk);
                let mut out = String::new();
                let mut consumed_stop = false;

                for line in text.split('\n') {
                    let line = line.trim();
                    if !line.starts_with("data:") {
                        continue;
                    }
                    let payload = line[5..].trim();
                    if payload == "[DONE]" {
                        consumed_stop = true;
                        continue;
                    }
                    let value: serde_json::Value = match serde_json::from_str(payload) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let choice = &value["choices"][0];
                    let delta = &choice["delta"];

                    // Testo: accumula in un unico blocco finché il tipo non cambia
                    if let Some(txt) = delta["content"].as_str() {
                        if !txt.is_empty() {
                            if self.current_block.as_deref() != Some("text") {
                                self.close_block(&mut out);
                                out.push_str(&self.block_start_text(self.block_index));
                                self.current_block = Some("text".to_string());
                            }
                            out.push_str(&self.text_delta(self.block_index, txt));
                            self.output_tokens += (txt.chars().count() / 4).max(1) as u64;
                        }
                    }

                    // Reasoning: blocco thinking (modelli che emettono reasoning prima del testo)
                    if let Some(r) = delta["reasoning"].as_str() {
                        if !r.is_empty() {
                            if self.current_block.as_deref() != Some("thinking") {
                                self.close_block(&mut out);
                                out.push_str(&self.block_start_thinking(self.block_index));
                                self.current_block = Some("thinking".to_string());
                            }
                            out.push_str(&self.thinking_delta(self.block_index, r));
                            self.output_tokens += (r.chars().count() / 4).max(1) as u64;
                        }
                    }

                    // Tool calls (possono arrivare frammentati)
                    if let Some(tc) = delta["tool_calls"].as_array() {
                        for call in tc {
                            let fn_name = call["function"]["name"].as_str();
                            let fn_args = call["function"]["arguments"].as_str().unwrap_or("");
                            let call_id = call["id"].as_str().unwrap_or("");

                            if self.current_block.as_deref() != Some("tool") {
                                self.close_block(&mut out);
                                if let Some(name) = fn_name {
                                    out.push_str(&self.block_start_tool(self.block_index, call_id, name));
                                } else {
                                    out.push_str(&self.block_start_tool(self.block_index, "toolu_unknown", "tool"));
                                }
                                self.current_block = Some("tool".to_string());
                                self.block_index += 1;
                            }
                            if !fn_args.is_empty() {
                                out.push_str(&self.tool_delta(self.block_index.saturating_sub(1), fn_args));
                                self.output_tokens += (fn_args.chars().count() / 4).max(1) as u64;
                            }
                        }
                    }

                    // finish_reason → stop_reason
                    if let Some(fr) = choice["finish_reason"].as_str() {
                        if !fr.is_empty() && fr != "null" {
                            let stop_reason = match fr {
                                "tool_calls" => "tool_use",
                                "length" => "max_tokens",
                                "stop" => "end_turn",
                                _ => "end_turn",
                            };
                            self.close_block(&mut out);
                            out.push_str(&self.message_delta(stop_reason));
                            out.push_str(&self.message_stop());
                            self.finished = true;
                        }
                    }
                }

                if out.is_empty() && !consumed_stop {
                    return self.poll_next(cx);
                }

                if out.is_empty() && consumed_stop && !self.finished {
                    self.close_block(&mut out);
                    out.push_str(&self.message_delta("end_turn"));
                    out.push_str(&self.message_stop());
                    self.finished = true;
                }

                Poll::Ready(Some(Ok(Bytes::from(out))))
            }
            Poll::Ready(Some(Err(e))) => {
                let mut out = String::new();
                if !self.finished {
                    self.close_block(&mut out);
                    out.push_str(&self.message_delta("end_turn"));
                    out.push_str(&self.message_stop());
                    self.finished = true;
                }
                out.push_str(&format!(
                    "event: error\ndata: {{\"type\":\"error\",\"error\":{{\"type\":\"overloaded_error\",\"message\":\"upstream error: {}\"}}}}\n\n",
                    e
                ));
                Poll::Ready(Some(Ok(Bytes::from(out))))
            }
            Poll::Ready(None) => {
                if !self.finished {
                    let mut out = String::new();
                    self.close_block(&mut out);
                    out.push_str(&self.message_delta("end_turn"));
                    out.push_str(&self.message_stop());
                    self.finished = true;
                    return Poll::Ready(Some(Ok(Bytes::from(out))));
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
