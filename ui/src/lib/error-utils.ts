export interface ErrorDisplay {
  message: string;
  details?: string;
}

export function getErrorDisplay(error: unknown): ErrorDisplay {
  if (error instanceof TypeError) {
    const message = error.message.toLowerCase();
    if (
      message.includes('fetch') ||
      message.includes('network') ||
      message.includes('failed to fetch')
    ) {
      return {
        message: 'Unable to connect to Penny server. Make sure it is running.',
        details: error.message,
      };
    }
  }

  if (error instanceof Error) {
    const message = error.message.toLowerCase();
    if (
      message.includes('invalid_type') ||
      message.includes('expected object')
    ) {
      return {
        message:
          'Received an unexpected response from Penny server. Make sure it is running.',
        details: error.message,
      };
    }
    if (message.includes('connection refused')) {
      return {
        message: 'Unable to connect to Penny server. Make sure it is running.',
        details: error.message,
      };
    }
    return { message: error.message };
  }

  return { message: 'An unknown error occurred.' };
}
