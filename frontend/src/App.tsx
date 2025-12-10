import { Router, Route } from '@solidjs/router';
import { Home } from './components/Home';
import { PdfViewer } from './components/PdfViewer';

export function App() {
  return (
    <Router>
      <Route path="/" component={Home} />
      <Route path="/pdf/:id" component={PdfViewer} />
    </Router>
  );
}
