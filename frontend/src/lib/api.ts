import type { Message, ChatResponse } from '../types';

const API_BASE = 'http://localhost:3001';

export interface DocumentMetadata {
  id: string;
  filename: string;
  keywords: string[];
  topics: string[];
  uploaded_at: string;
}

export async function uploadPdf(file: File): Promise<string> {
  const formData = new FormData();
  formData.append('pdf', file);

  const response = await fetch(`${API_BASE}/api/upload`, {
    method: 'POST',
    body: formData,
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || 'Upload failed');
  }

  const data = await response.json();
  return data.document_id;
}

export async function sendChatMessage(
  documentId: string,
  messages: Message[]
): Promise<ChatResponse> {
  const response = await fetch(`${API_BASE}/api/chat`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      document_id: documentId,
      messages: messages.map(m => ({
        role: m.role,
        content: m.content,
      })),
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || 'Chat request failed');
  }

  return response.json();
}

export async function getChatHistory(documentId: string): Promise<Message[]> {
  const response = await fetch(`${API_BASE}/api/chat/history/${documentId}`);

  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || 'Failed to load chat history');
  }

  const data = await response.json();
  return data.map((msg: any) => ({
    id: msg.id,
    role: msg.role,
    content: msg.content,
    timestamp: new Date(msg.created_at),
  }));
}

export async function getRecentDocuments(limit: number = 20): Promise<DocumentMetadata[]> {
  const response = await fetch(`${API_BASE}/api/documents?limit=${limit}`);

  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || 'Failed to load documents');
  }

  return response.json();
}
