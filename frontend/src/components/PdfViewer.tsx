import { useParams } from '@solidjs/router';
import { ChatWidget } from './ChatWidget';

export function PdfViewer() {
  const params = useParams();
  const documentId = () => params.id;

  const pdfUrl = () =>
    `https://mozilla.github.io/pdf.js/web/viewer.html?file=${encodeURIComponent(
      `http://localhost:3001/api/documents/${documentId()}`
    )}&sidebarViewOnLoad=0`;

  return (
    <div class="viewer-container">
      <iframe id="pdfFrame" src={pdfUrl()} title="PDF Viewer" class="pdf-iframe" />
      <ChatWidget documentId={documentId()} />
    </div>
  );
}
