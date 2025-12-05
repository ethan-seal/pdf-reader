import { createMemo } from 'solid-js';
import { processPageReferences, handlePageNavigation } from '../lib/pageLinks';

interface MarkdownRendererProps {
  content: string;
}

export function MarkdownRenderer(props: MarkdownRendererProps) {
  const processedContent = createMemo(() => processPageReferences(props.content));

  const handleClick = (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.tagName === 'A') {
      const href = target.getAttribute('href');
      const pageMatch = href?.match(/^#page-(\d+)$/);

      if (pageMatch) {
        e.preventDefault();
        handlePageNavigation(pageMatch[1]);
      }
    }
  };

  return (
    <div class="markdown-content" onClick={handleClick} innerHTML={renderMarkdown(processedContent())} />
  );
}

function renderMarkdown(text: string): string {
  return text
    // Code blocks with optional language identifier (must be processed before inline code)
    .replace(/```(\w+)?\n([\s\S]*?)```/g, (_, lang, code) => {
      const language = lang ? ` class="language-${lang}"` : '';
      return `<pre><code${language}>${escapeHtml(code.trim())}</code></pre>`;
    })
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2">$1</a>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
    .replace(/\n/g, '<br>');
}

function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}
