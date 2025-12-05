export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  isError?: boolean;
  retryContext?: {
    userMessageId: string;
  };
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
