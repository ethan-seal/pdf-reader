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

export interface Usage {
  input_tokens: number;
  output_tokens: number;
  cache_creation_input_tokens?: number;
  cache_read_input_tokens?: number;
}

export interface ChatResponse {
  response: string;
  usage?: Usage;
}
