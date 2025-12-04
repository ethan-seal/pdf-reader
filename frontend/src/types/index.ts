export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export interface ChatRequest {
  document_id: string;
  messages: Array<{
    role: string;
    content: string;
  }>;
}

export interface ChatResponse {
  response: string;
  usage?: {
    input_tokens: number;
    output_tokens: number;
  };
}
