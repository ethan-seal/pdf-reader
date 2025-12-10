import { createSignal, onMount, For, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { uploadPdf, getRecentDocuments, type DocumentMetadata } from '../lib/api';

export function Home() {
  const [documents, setDocuments] = createSignal<DocumentMetadata[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [uploading, setUploading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const navigate = useNavigate();

  onMount(async () => {
    try {
      const docs = await getRecentDocuments(20);
      setDocuments(docs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load documents');
    } finally {
      setLoading(false);
    }
  });

  const handleFileChange = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];

    if (!file) return;

    if (file.type !== 'application/pdf') {
      setError('Please select a PDF file');
      return;
    }

    setUploading(true);
    setError(null);

    try {
      const documentId = await uploadPdf(file);
      navigate(`/pdf/${documentId}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Upload failed');
    } finally {
      setUploading(false);
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;

    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
    });
  };

  return (
    <div class="home-container">
      <header class="home-header">
        <h1>PDF Research Reader</h1>
        <label class="upload-button">
          {uploading() ? 'Uploading...' : 'Upload PDF'}
          <input
            type="file"
            accept=".pdf"
            onChange={handleFileChange}
            disabled={uploading()}
            style={{ display: 'none' }}
          />
        </label>
      </header>

      {error() && <p class="error">{error()}</p>}

      <Show when={loading()}>
        <div class="loading">Loading...</div>
      </Show>

      <Show when={!loading() && documents().length === 0}>
        <div class="empty-state">
          <p>No PDFs uploaded yet.</p>
          <p>Upload your first research paper to get started!</p>
        </div>
      </Show>

      <Show when={!loading() && documents().length > 0}>
        <div class="documents-grid">
          <For each={documents()}>
            {(doc) => (
              <div class="document-card" onClick={() => navigate(`/pdf/${doc.id}`)}>
                <div class="document-header">
                  <h3 class="document-title">{doc.filename}</h3>
                  <span class="document-date">{formatDate(doc.uploaded_at)}</span>
                </div>

                <Show when={doc.topics.length > 0}>
                  <div class="document-section">
                    <h4>Topics</h4>
                    <div class="tags">
                      <For each={doc.topics}>
                        {(topic) => <span class="tag topic-tag">{topic}</span>}
                      </For>
                    </div>
                  </div>
                </Show>

                <Show when={doc.keywords.length > 0}>
                  <div class="document-section">
                    <h4>Keywords</h4>
                    <div class="tags">
                      <For each={doc.keywords}>
                        {(keyword) => <span class="tag keyword-tag">{keyword}</span>}
                      </For>
                    </div>
                  </div>
                </Show>

                <Show when={doc.topics.length === 0 && doc.keywords.length === 0}>
                  <div class="document-section">
                    <p class="no-metadata">Metadata extraction in progress...</p>
                  </div>
                </Show>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
