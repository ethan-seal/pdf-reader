export function processPageReferences(content: string): string {
  return content.replace(
    /\(\s*page\s+(\d+(?:\s*,\s*page\s+\d+)*)\s*\)/g,
    (match, pageList: string) => {
      const pages = pageList.split(/\s*,\s*page\s+/);
      const links = pages.map(
        (pageNum: string) => `[page ${pageNum.trim()}](#page-${pageNum.trim()})`
      );
      return `(${links.join(', ')})`;
    }
  );
}

export function handlePageNavigation(pageNum: string): void {
  const pdfFrame = document.getElementById('pdfFrame') as HTMLIFrameElement;
  if (pdfFrame?.src) {
    const baseUrl = pdfFrame.src.split('#')[0];
    pdfFrame.src = `${baseUrl}#page=${pageNum}`;
  }
}
