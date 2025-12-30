/**
 * Utility functions for Markdown processing
 */

/**
 * Convert Markdown to plain text (for previews)
 * Removes markdown syntax and returns plain text
 * This function preserves the text content while removing formatting
 */
export function markdownToPlainText(markdown: string): string {
  if (!markdown) return '';
  
  let text = markdown;
  
  // Remove code blocks (but keep a placeholder if needed)
  text = text.replace(/```[\s\S]*?```/g, '');
  
  // Remove inline code but keep the content
  text = text.replace(/`([^`]+)`/g, '$1');
  
  // Remove images completely
  text = text.replace(/!\[([^\]]*)\]\([^\)]+\)/g, '');
  
  // Convert links to just the text
  text = text.replace(/\[([^\]]+)\]\([^\)]+\)/g, '$1');
  
  // Remove headers but keep the text
  text = text.replace(/^#{1,6}\s+(.+)$/gm, '$1');
  
  // Remove bold/italic but keep the text
  text = text.replace(/\*\*([^*]+)\*\*/g, '$1');
  text = text.replace(/\*([^*]+)\*/g, '$1');
  text = text.replace(/__([^_]+)__/g, '$1');
  text = text.replace(/_([^_]+)_/g, '$1');
  
  // Remove strikethrough but keep the text
  text = text.replace(/~~([^~]+)~~/g, '$1');
  
  // Remove task list markers ([x] and [ ]) but keep the text
  text = text.replace(/\[x\]/gi, '');
  text = text.replace(/\[\s\]/g, '');
  
  // Remove list markers but keep the text (convert to plain text)
  text = text.replace(/^[\s]*[-*+]\s+/gm, '');
  text = text.replace(/^[\s]*\d+\.\s+/gm, '');
  
  // Remove blockquote markers but keep the text
  text = text.replace(/^>\s+/gm, '');
  
  // Remove horizontal rules
  text = text.replace(/^---+$/gm, '');
  text = text.replace(/^\*\*\*+$/gm, '');
  text = text.replace(/^___+$/gm, '');
  
  // Remove HTML tags
  text = text.replace(/<[^>]*>/g, '');
  
  // Normalize whitespace (multiple spaces/newlines to single space)
  text = text.replace(/\s+/g, ' ');
  
  // Trim whitespace
  return text.trim();
}

