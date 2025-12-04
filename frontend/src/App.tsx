import { Router, Route } from '@solidjs/router';
import { Upload } from './components/Upload';
import { PdfViewer } from './components/PdfViewer';

export function App() {
  return (
    <Router>
      <Route path="/" component={Upload} />
      <Route path="/pdf/:id" component={PdfViewer} />
    </Router>
  );
}
