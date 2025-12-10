import { For, Show, createEffect, createMemo, createSignal, type Accessor } from 'solid-js';
import { processPageReferences, handlePageNavigation } from '../lib/pageLinks';

interface MarkdownRendererProps {
  content: string;
}

interface MarkdownSection {
  id: string;
  title: string;
  level: number;
  content: string;
  children: MarkdownSection[];
  parentId?: string;
}

export function MarkdownRenderer(props: MarkdownRendererProps) {
  const processedContent = createMemo(() => processPageReferences(props.content));
  const sections = createMemo(() => parseMarkdownSections(processedContent()));
  const [collapsedSections, setCollapsedSections] = createSignal<Set<string>>(new Set());

  createEffect(() => {
    const collectIds = (nodes: MarkdownSection[]): string[] => {
      return nodes.flatMap((node) => [node.id, ...collectIds(node.children)]);
    };

    const ids = collectIds(sections());

    setCollapsedSections((prev) => {
      const next = new Set<string>();

      ids.forEach((id) => {
        if (prev.has(id)) {
          next.add(id);
        }
      });

      // Preserve reference when nothing changed to avoid extra renders
      if (next.size === prev.size && ids.every((id) => prev.has(id))) {
        return prev;
      }

      return next;
    });
  });

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
    <div class="markdown-content" onClick={handleClick}>
      <Show when={sections().some((section) => section.title)} fallback={
        <div innerHTML={renderMarkdown(processedContent())} />
      }>
        <SectionList
          sections={sections()}
          collapsedSections={collapsedSections}
          onToggle={(id) => {
            setCollapsedSections((prev) => {
              const next = new Set(prev);
              if (next.has(id)) {
                next.delete(id);
              } else {
                next.add(id);
              }
              return next;
            });
          }}
        />
      </Show>
    </div>
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

function parseMarkdownSections(text: string): MarkdownSection[] {
  const lines = text.split(/\r?\n/);
  const root: MarkdownSection = {
    id: 'root',
    title: '',
    level: 0,
    content: '',
    children: [],
  };

  let current: MarkdownSection = root;
  const stack: MarkdownSection[] = [root];
  let sectionIndex = 0;
  let inCodeBlock = false;

  for (const line of lines) {
    const trimmed = line.trim();
    const isCodeFence = trimmed.startsWith('```');

    if (!inCodeBlock) {
      const headerMatch = line.match(/^(#{1,6})\s+(.*)$/);
      if (headerMatch) {
        const level = headerMatch[1].length;
        const title = headerMatch[2].trim();
        const newSection: MarkdownSection = {
          id: `section-${sectionIndex++}`,
          title,
          level,
          content: '',
          children: [],
          parentId: undefined,
        };

        while (stack.length > 0 && stack[stack.length - 1].level >= level) {
          stack.pop();
        }

        const parent = stack[stack.length - 1] || root;
        newSection.parentId = parent.id;
        parent.children.push(newSection);

        stack.push(newSection);
        current = newSection;
        continue;
      }
    }

    current.content = current.content ? `${current.content}\n${line}` : line;

    if (isCodeFence) {
      inCodeBlock = !inCodeBlock;
    }
  }

  return root.children.length > 0 ? root.children : [root];
}

interface SectionListProps {
  sections: MarkdownSection[];
  collapsedSections: Accessor<Set<string>>;
  onToggle: (id: string) => void;
  ancestorCollapsed?: boolean;
}

function SectionList(props: SectionListProps) {
  return (
    <For each={props.sections}>
      {(section) => (
        <Section
          section={section}
          collapsedSections={props.collapsedSections}
          onToggle={props.onToggle}
          ancestorCollapsed={props.ancestorCollapsed ?? false}
        />
      )}
    </For>
  );
}

interface SectionProps {
  section: MarkdownSection;
  collapsedSections: Accessor<Set<string>>;
  onToggle: (id: string) => void;
  ancestorCollapsed: boolean;
}

function Section(props: SectionProps) {
  const collapsedHere = createMemo(() => props.collapsedSections().has(props.section.id));
  const isHidden = createMemo(() => props.ancestorCollapsed);
  const bodyVisible = createMemo(() => !collapsedHere() && !isHidden());
  const childAncestorCollapsed = createMemo(() => isHidden() || collapsedHere());

  if (!props.section.title) {
    if (isHidden()) return null;
    return <div class="section-body standalone" innerHTML={renderMarkdown(props.section.content)} />;
  }

  return (
    <div class={`collapsible-section level-${Math.min(props.section.level, 3)} ${collapsedHere() ? 'collapsed' : ''}`}>
      {!isHidden() && (
        <button type="button" class="section-header" onClick={() => props.onToggle(props.section.id)}>
          <span class={`chevron ${collapsedHere() ? '' : 'open'}`}>▸</span>
          <span class="section-title">{props.section.title}</span>
        </button>
      )}
      <Show when={bodyVisible()}>
        <div class="section-body" innerHTML={renderMarkdown(props.section.content)} />
      </Show>
      <Show when={props.section.children.length > 0 && !isHidden()}>
        <SectionList
          sections={props.section.children}
          collapsedSections={props.collapsedSections}
          onToggle={props.onToggle}
          ancestorCollapsed={childAncestorCollapsed()}
        />
      </Show>
    </div>
  );
}
