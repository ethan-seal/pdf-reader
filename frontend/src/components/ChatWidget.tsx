import { createSignal, For, onMount, createEffect } from 'solid-js';
import type { Message } from '../types';
import { sendChatMessage, getChatHistory } from '../lib/api';
import { MarkdownRenderer } from './MarkdownRenderer';

interface ChatWidgetProps {
  documentId: string;
}

export function ChatWidget(props: ChatWidgetProps) {
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [input, setInput] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [collapsed, setCollapsed] = createSignal(false);

  let messagesEndRef: HTMLDivElement | undefined;

  const scrollToBottom = () => {
    messagesEndRef?.scrollIntoView({ behavior: 'smooth' });
  };

  createEffect(() => {
    if (messages().length > 0) {
      setTimeout(scrollToBottom, 100);
    }
  });

  const sendMessage = async (content: string, userMessageId?: string) => {
    if (!content.trim() || loading()) return;

    const messageId = userMessageId || Date.now().toString();
    const userMessage: Message = {
      id: messageId,
      role: 'user',
      content: content.trim(),
      timestamp: new Date(),
    };

    if (!userMessageId) {
      setMessages((prev) => [...prev, userMessage]);
      setInput('');
    }
    setLoading(true);

    try {
      const response = await sendChatMessage(props.documentId, [...messages(), userMessage]);

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response.response,
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error) {
      console.error('Chat error:', error);

      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: error instanceof Error ? `Error: ${error.message}` : 'Failed to get response',
        timestamp: new Date(),
        isError: true,
        retryContext: {
          userMessageId: messageId,
        },
      };

      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setLoading(false);
    }
  };

  const retryMessage = (errorMessageId: string) => {
    const errorMsg = messages().find(m => m.id === errorMessageId);
    if (!errorMsg?.retryContext) return;

    const userMsg = messages().find(m => m.id === errorMsg.retryContext!.userMessageId);
    if (!userMsg) return;

    setMessages((prev) => prev.filter(m => m.id !== errorMessageId));
    sendMessage(userMsg.content, userMsg.id);
  };

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    sendMessage(input());
  };

  onMount(async () => {
    try {
      // Load chat history first
      const history = await getChatHistory(props.documentId);

      if (history.length > 0) {
        // If history exists, load it
        setMessages(history);
      } else {
        // If no history, send initial greeting
        sendMessage(
          'Please provide a brief overview of this paper and suggest some questions I might ask.'
        );
      }
    } catch (error) {
      console.error('Failed to load chat history:', error);
      // If history loading fails, send initial greeting
      sendMessage(
        'Please provide a brief overview of this paper and suggest some questions I might ask.'
      );
    }
  });

  return (
    <div class={`chat-widget ${collapsed() ? 'collapsed' : ''}`}>
      <div class="chat-header">
        <h2>AI Assistant</h2>
        <button
          class="collapse-toggle"
          onClick={() => setCollapsed(!collapsed())}
          title={collapsed() ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {collapsed() ? '◀' : '▶'}
        </button>
      </div>

      <div class="chat-messages">
        <For each={messages()}>
          {(message) => (
            <div class={`message ${message.role} ${message.isError ? 'error' : ''}`}>
              {message.role === 'assistant' ? (
                <MarkdownRenderer content={message.content} />
              ) : (
                <p>{message.content}</p>
              )}
              {message.isError && (
                <button
                  class="retry-button"
                  onClick={() => retryMessage(message.id)}
                  disabled={loading()}
                >
                  Retry
                </button>
              )}
            </div>
          )}
        </For>

        {loading() && (
          <div class="message assistant loading">
            <span class="loading-dots">●●●</span>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <form class="chat-input" onSubmit={handleSubmit}>
        <input
          type="text"
          value={input()}
          onInput={(e) => setInput(e.currentTarget.value)}
          placeholder={loading() ? 'AI is thinking...' : 'Ask a question...'}
          disabled={loading()}
        />
        <button type="submit" disabled={loading() || !input().trim()}>
          Send
        </button>
      </form>
    </div>
  );
}
