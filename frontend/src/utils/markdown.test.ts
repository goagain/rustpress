import { describe, it, expect } from 'vitest';
import { markdownToPlainText } from './markdown';

describe('markdownToPlainText', () => {
  describe('List conversion (bullet points)', () => {
    it('should convert unordered list items with - to plain text', () => {
      const markdown = '- First item\n- Second item\n- Third item';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('First item Second item Third item');
    });

    it('should convert unordered list items with * to plain text', () => {
      const markdown = '* First item\n* Second item\n* Third item';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('First item Second item Third item');
    });

    it('should convert unordered list items with + to plain text', () => {
      const markdown = '+ First item\n+ Second item\n+ Third item';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('First item Second item Third item');
    });

    it('should convert ordered list items to plain text', () => {
      const markdown = '1. First item\n2. Second item\n3. Third item';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('First item Second item Third item');
    });

    it('should handle nested lists', () => {
      const markdown = '- Parent item\n  - Child item 1\n  - Child item 2';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Parent item Child item 1 Child item 2');
    });

    it('should handle mixed list types', () => {
      const markdown = '- Unordered item\n1. Ordered item\n* Another unordered';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Unordered item Ordered item Another unordered');
    });
  });

  describe('Headers', () => {
    it('should convert H1 headers to plain text', () => {
      const markdown = '# Main Title';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Main Title');
    });

    it('should convert H2 headers to plain text', () => {
      const markdown = '## Subtitle';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Subtitle');
    });

    it('should convert all header levels (H1-H6)', () => {
      const markdown = '# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('H1 H2 H3 H4 H5 H6');
    });
  });

  describe('Text formatting', () => {
    it('should remove bold formatting but keep text', () => {
      const markdown = 'This is **bold** text';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('This is bold text');
    });

    it('should remove italic formatting but keep text', () => {
      const markdown = 'This is *italic* text';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('This is italic text');
    });

    it('should remove underline formatting but keep text', () => {
      const markdown = 'This is __underline__ text';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('This is underline text');
    });

    it('should remove strikethrough formatting but keep text', () => {
      const markdown = 'This is ~~strikethrough~~ text';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('This is strikethrough text');
    });

    it('should handle mixed formatting', () => {
      const markdown = 'This is **bold** and *italic* text';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('This is bold and italic text');
    });
  });

  describe('Links and images', () => {
    it('should convert links to plain text (keep link text)', () => {
      const markdown = 'Check out [this link](https://example.com)';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Check out this link');
    });

    it('should remove images completely', () => {
      const markdown = 'Here is an image: ![alt text](image.jpg)';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Here is an image:');
    });

    it('should handle multiple links', () => {
      const markdown = '[Link 1](url1) and [Link 2](url2)';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Link 1 and Link 2');
    });
  });

  describe('Code blocks', () => {
    it('should remove code blocks completely', () => {
      const markdown = 'Here is code:\n```javascript\nconst x = 1;\n```\nEnd';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Here is code: End');
    });

    it('should remove inline code but keep content', () => {
      const markdown = 'Use `console.log()` to debug';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Use console.log() to debug');
    });

    it('should handle multiple code blocks', () => {
      const markdown = 'Code 1:\n```\ncode1\n```\nCode 2:\n```\ncode2\n```';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Code 1: Code 2:');
    });
  });

  describe('Blockquotes', () => {
    it('should remove blockquote markers but keep text', () => {
      const markdown = '> This is a quote';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('This is a quote');
    });

    it('should handle multi-line blockquotes', () => {
      const markdown = '> Line 1\n> Line 2\n> Line 3';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Line 1 Line 2 Line 3');
    });
  });

  describe('Horizontal rules', () => {
    it('should remove horizontal rules (---)', () => {
      const markdown = 'Text before\n---\nText after';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Text before Text after');
    });

    it('should remove horizontal rules (***)', () => {
      const markdown = 'Text before\n***\nText after';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Text before Text after');
    });

    it('should remove horizontal rules (___)', () => {
      const markdown = 'Text before\n___\nText after';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Text before Text after');
    });
  });

  describe('HTML tags', () => {
    it('should remove HTML tags', () => {
      const markdown = 'Text with <strong>HTML</strong> tags';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Text with HTML tags');
    });

    it('should handle self-closing tags', () => {
      const markdown = 'Text with <br /> break';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Text with break');
    });
  });

  describe('Whitespace normalization', () => {
    it('should normalize multiple spaces to single space', () => {
      const markdown = 'Text    with    multiple    spaces';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Text with multiple spaces');
    });

    it('should normalize newlines to single space', () => {
      const markdown = 'Line 1\n\n\nLine 2';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Line 1 Line 2');
    });

    it('should trim leading and trailing whitespace', () => {
      const markdown = '   Text with spaces   ';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Text with spaces');
    });
  });

  describe('Task lists', () => {
    it('should remove task list markers but keep text', () => {
      const markdown = '- [x] Completed task\n- [ ] Incomplete task';
      const result = markdownToPlainText(markdown);
      expect(result).toContain('Completed task');
      expect(result).toContain('Incomplete task');
      expect(result).not.toContain('[x]');
      expect(result).not.toContain('[ ]');
    });

    it('should handle task lists with nested content', () => {
      const markdown = '- [x] Task with **bold** text';
      const result = markdownToPlainText(markdown);
      expect(result).toContain('Task with bold text');
      expect(result).not.toContain('[x]');
      expect(result).not.toContain('**');
    });
  });

  describe('Edge cases', () => {
    it('should handle empty string', () => {
      const result = markdownToPlainText('');
      expect(result).toBe('');
    });

    it('should handle null/undefined (returns empty string)', () => {
      const result1 = markdownToPlainText(null as any);
      const result2 = markdownToPlainText(undefined as any);
      expect(result1).toBe('');
      expect(result2).toBe('');
    });

    it('should handle plain text without markdown', () => {
      const markdown = 'Just plain text without any formatting';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('Just plain text without any formatting');
    });

    it('should handle markdown with only whitespace', () => {
      const markdown = '   \n\n   ';
      const result = markdownToPlainText(markdown);
      expect(result).toBe('');
    });

    it('should handle complex markdown with all features', () => {
      const markdown = `# Title

This is **bold** and *italic* text.

- First item
- Second item
- Third item

\`\`\`javascript
const code = 'example';
\`\`\`

[Link text](https://example.com)

> This is a quote`;

      const result = markdownToPlainText(markdown);
      expect(result).toContain('Title');
      expect(result).toContain('bold');
      expect(result).toContain('italic');
      expect(result).toContain('First item');
      expect(result).toContain('Second item');
      expect(result).toContain('Third item');
      expect(result).toContain('Link text');
      expect(result).toContain('This is a quote');
      expect(result).not.toContain('```');
      expect(result).not.toContain('**');
      expect(result).not.toContain('*');
      expect(result).not.toContain('-');
    });

    it('should handle markdown with special characters', () => {
      const markdown = 'Text with special chars: < > & " \'';
      const result = markdownToPlainText(markdown);
      expect(result).toContain('Text with special chars');
    });

    it('should handle markdown with emoji', () => {
      const markdown = 'Text with emoji: ✅ 🚀 🎉';
      const result = markdownToPlainText(markdown);
      expect(result).toContain('Text with emoji');
      expect(result).toContain('✅');
      expect(result).toContain('🚀');
      expect(result).toContain('🎉');
    });

    it('should handle very long markdown content', () => {
      const longContent = Array(100).fill('- List item').join('\n');
      const result = markdownToPlainText(longContent);
      expect(result.split('List item').length - 1).toBe(100);
      expect(result).not.toContain('-');
    });
  });
});

