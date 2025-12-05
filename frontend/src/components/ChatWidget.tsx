import { createSignal, For, onMount, createEffect } from 'solid-js';
import type { Message, Usage } from '../types';
import { sendChatMessage, getChatHistory } from '../lib/api';
import { MarkdownRenderer } from './MarkdownRenderer';

interface ChatWidgetProps {
  documentId: string;
}

// Claude Sonnet 4.5 pricing (per million tokens)
const PRICING = {
  INPUT: 3.00,
  OUTPUT: 15.00,
  CACHE_WRITE: 3.75,
  CACHE_READ: 0.30,
};

function calculateCost(usage: Usage): number {
  const inputCost = (usage.input_tokens / 1_000_000) * PRICING.INPUT;
  const outputCost = (usage.output_tokens / 1_000_000) * PRICING.OUTPUT;
  const cacheWriteCost = ((usage.cache_creation_input_tokens || 0) / 1_000_000) * PRICING.CACHE_WRITE;
  const cacheReadCost = ((usage.cache_read_input_tokens || 0) / 1_000_000) * PRICING.CACHE_READ;

  return inputCost + outputCost + cacheWriteCost + cacheReadCost;
}

const MIN_WIDTH = 300;
const MAX_WIDTH = 800;
const COLLAPSE_THRESHOLD = 200;
const DEFAULT_WIDTH = 400;

export function ChatWidget(props: ChatWidgetProps) {
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [input, setInput] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [collapsed, setCollapsed] = createSignal(false);
  const [sessionUsage, setSessionUsage] = createSignal<Usage[]>([]);
  const [width, setWidth] = createSignal(DEFAULT_WIDTH);
  const [isResizing, setIsResizing] = createSignal(false);

  let messagesEndRef: HTMLDivElement | undefined;
  let chatWidgetRef: HTMLDivElement | undefined;

  const totalCost = () => {
    return sessionUsage().reduce((total, usage) => total + calculateCost(usage), 0);
  };

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

      // Track usage for this request
      if (response.usage) {
        setSessionUsage((prev) => [...prev, response.usage!]);
      }

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

  const handleMouseDown = (e: MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizing()) return;

    // Calculate new width based on mouse position from right edge
    const newWidth = window.innerWidth - e.clientX;

    if (newWidth < COLLAPSE_THRESHOLD) {
      // Auto-collapse if dragged too narrow
      setCollapsed(true);
      setWidth(DEFAULT_WIDTH); // Reset to default for when it's expanded again
    } else if (newWidth >= MIN_WIDTH && newWidth <= MAX_WIDTH) {
      setCollapsed(false);
      setWidth(newWidth);
    } else if (newWidth > MAX_WIDTH) {
      setCollapsed(false);
      setWidth(MAX_WIDTH);
    }
  };

  const handleMouseUp = () => {
    setIsResizing(false);
  };

  onMount(async () => {
    // Add global mouse event listeners for resize
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

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

    // Cleanup on unmount
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  });

  return (
    <div
      ref={chatWidgetRef}
      class={`chat-widget ${collapsed() ? 'collapsed' : ''} ${isResizing() ? 'resizing' : ''}`}
      style={{ width: collapsed() ? '50px' : `${width()}px` }}
    >
      <div
        class="resize-handle"
        onMouseDown={handleMouseDown}
        title="Drag to resize"
      />
      <div class="chat-header">
        <div class="header-content">
          <h2>AI Assistant</h2>
          {totalCost() > 0 && (
            <div class="cost-display" title="Total API cost for this session">
              ${totalCost().toFixed(4)}
            </div>
          )}
        </div>
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
