import { createSignal } from 'solid-js';
import { uploadPdf } from '../lib/api';
import { useNavigate } from '@solidjs/router';

export function Upload() {
  const [uploading, setUploading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const navigate = useNavigate();

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

  return (
    <div class="upload-container">
      <h1>PDF Research Reader</h1>
      <p>Upload a research paper to get started</p>

      <label class="upload-button">
        {uploading() ? 'Uploading...' : 'Choose PDF'}
        <input
          type="file"
          accept=".pdf"
          onChange={handleFileChange}
          disabled={uploading()}
          style={{ display: 'none' }}
        />
      </label>

      {error() && <p class="error">{error()}</p>}
    </div>
  );
}
