import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeHighlight from 'rehype-highlight';

describe('PostDetail Markdown Rendering', () => {
  const renderMarkdown = (content: string) => {
    return render(
      <div className="prose prose-slate prose-lg max-w-none">
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          rehypePlugins={[rehypeRaw, rehypeHighlight]}
          components={{
            ul: ({node, className, ...props}) => (
              <ul className={`${className || ''} list-disc pl-6`} {...props} />
            ),
            ol: ({node, className, ...props}) => (
              <ol className={`${className || ''} list-decimal pl-6`} {...props} />
            ),
            li: ({node, className, ...props}) => (
              <li className={`${className || ''} my-1`} {...props} />
            ),
            table: ({node, className, ...props}) => (
              <table className={`${className || ''} border border-slate-300 w-full`} {...props} />
            ),
            thead: ({node, className, ...props}) => (
              <thead className={`${className || ''} bg-slate-50`} {...props} />
            ),
            tbody: ({node, className, ...props}) => (
              <tbody className={className || ''} {...props} />
            ),
            tr: ({node, className, ...props}) => (
              <tr className={`${className || ''} border-b border-slate-200`} {...props} />
            ),
            th: ({node, className, ...props}) => (
              <th className={`${className || ''} bg-slate-100 font-semibold p-3 text-left border border-slate-300`} {...props} />
            ),
            td: ({node, className, ...props}) => (
              <td className={`${className || ''} p-3 border border-slate-200`} {...props} />
            ),
            input: ({node, className, ...props}: any) => {
              if (props.type === 'checkbox') {
                return (
                  <input
                    type="checkbox"
                    className={`${className || ''} mr-2`}
                    disabled
                    {...props}
                  />
                );
              }
              return <input className={className || ''} {...props} />;
            },
          }}
        >
          {content}
        </ReactMarkdown>
      </div>
    );
  };

  describe('List rendering (bullet points)', () => {
    it('should render unordered list with - as bullet points', () => {
      const markdown = '- First item\n- Second item\n- Third item';
      renderMarkdown(markdown);
      
      const list = screen.getByRole('list');
      expect(list).toBeInTheDocument();
      
      const items = screen.getAllByRole('listitem');
      expect(items).toHaveLength(3);
      expect(items[0]).toHaveTextContent('First item');
      expect(items[1]).toHaveTextContent('Second item');
      expect(items[2]).toHaveTextContent('Third item');
    });

    it('should render unordered list with * as bullet points', () => {
      const markdown = '* First item\n* Second item\n* Third item';
      renderMarkdown(markdown);
      
      const list = screen.getByRole('list');
      expect(list).toBeInTheDocument();
      
      const items = screen.getAllByRole('listitem');
      expect(items).toHaveLength(3);
    });

    it('should render unordered list with + as bullet points', () => {
      const markdown = '+ First item\n+ Second item\n+ Third item';
      renderMarkdown(markdown);
      
      const list = screen.getByRole('list');
      expect(list).toBeInTheDocument();
      
      const items = screen.getAllByRole('listitem');
      expect(items).toHaveLength(3);
    });

    it('should render ordered list with numbers', () => {
      const markdown = '1. First item\n2. Second item\n3. Third item';
      renderMarkdown(markdown);
      
      const list = screen.getByRole('list');
      expect(list).toBeInTheDocument();
      expect(list.tagName.toLowerCase()).toBe('ol');
      
      const items = screen.getAllByRole('listitem');
      expect(items).toHaveLength(3);
    });

    it('should render nested lists', () => {
      const markdown = '- Parent item\n  - Child item 1\n  - Child item 2';
      renderMarkdown(markdown);
      
      const lists = screen.getAllByRole('list');
      expect(lists.length).toBeGreaterThan(1);
      
      const items = screen.getAllByRole('listitem');
      expect(items.length).toBeGreaterThanOrEqual(3);
    });

    it('should render task lists with checkboxes', () => {
      const markdown = '- [x] Completed task\n- [ ] Incomplete task';
      renderMarkdown(markdown);
      
      const checkboxes = document.querySelectorAll('input[type="checkbox"]');
      expect(checkboxes).toHaveLength(2);
      
      const completedCheckbox = checkboxes[0] as HTMLInputElement;
      const incompleteCheckbox = checkboxes[1] as HTMLInputElement;
      
      expect(completedCheckbox.checked).toBe(true);
      expect(incompleteCheckbox.checked).toBe(false);
      expect(completedCheckbox.disabled).toBe(true);
      expect(incompleteCheckbox.disabled).toBe(true);
    });

    it('should apply correct CSS classes to unordered lists', () => {
      const markdown = '- Item 1\n- Item 2';
      renderMarkdown(markdown);
      
      const list = screen.getByRole('list');
      expect(list).toHaveClass('list-disc');
      expect(list).toHaveClass('pl-6');
    });

    it('should apply correct CSS classes to ordered lists', () => {
      const markdown = '1. Item 1\n2. Item 2';
      renderMarkdown(markdown);
      
      const list = screen.getByRole('list');
      expect(list.tagName.toLowerCase()).toBe('ol');
      expect(list).toHaveClass('list-decimal');
      expect(list).toHaveClass('pl-6');
    });
  });

  describe('Headers', () => {
    it('should render H1 header', () => {
      const markdown = '# Main Title';
      renderMarkdown(markdown);
      
      const heading = screen.getByRole('heading', { level: 1 });
      expect(heading).toBeInTheDocument();
      expect(heading).toHaveTextContent('Main Title');
    });

    it('should render H2 header', () => {
      const markdown = '## Subtitle';
      renderMarkdown(markdown);
      
      const heading = screen.getByRole('heading', { level: 2 });
      expect(heading).toBeInTheDocument();
      expect(heading).toHaveTextContent('Subtitle');
    });

    it('should render all header levels (H1-H6)', () => {
      const markdown = '# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6';
      renderMarkdown(markdown);
      
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('H1');
      expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent('H2');
      expect(screen.getByRole('heading', { level: 3 })).toHaveTextContent('H3');
      expect(screen.getByRole('heading', { level: 4 })).toHaveTextContent('H4');
      expect(screen.getByRole('heading', { level: 5 })).toHaveTextContent('H5');
      expect(screen.getByRole('heading', { level: 6 })).toHaveTextContent('H6');
    });
  });

  describe('Text formatting', () => {
    it('should render bold text', () => {
      const markdown = 'This is **bold** text';
      renderMarkdown(markdown);
      
      const boldElement = screen.getByText('bold');
      expect(boldElement.tagName.toLowerCase()).toBe('strong');
    });

    it('should render italic text', () => {
      const markdown = 'This is *italic* text';
      renderMarkdown(markdown);
      
      const italicElement = screen.getByText('italic');
      expect(italicElement.tagName.toLowerCase()).toBe('em');
    });

    it('should render strikethrough text', () => {
      const markdown = 'This is ~~strikethrough~~ text';
      renderMarkdown(markdown);
      
      const strikeElement = screen.getByText('strikethrough');
      expect(strikeElement.tagName.toLowerCase()).toBe('del');
    });
  });

  describe('Links', () => {
    it('should render links correctly', () => {
      const markdown = 'Check out [this link](https://example.com)';
      renderMarkdown(markdown);
      
      const link = screen.getByRole('link');
      expect(link).toBeInTheDocument();
      expect(link).toHaveTextContent('this link');
      expect(link).toHaveAttribute('href', 'https://example.com');
    });
  });

  describe('Code blocks', () => {
    it('should render code blocks', () => {
      const markdown = '```javascript\nconst x = 1;\n```';
      renderMarkdown(markdown);
      
      const codeBlock = document.querySelector('pre code');
      expect(codeBlock).toBeInTheDocument();
      expect(codeBlock).toHaveTextContent('const x = 1;');
    });

    it('should render inline code', () => {
      const markdown = 'Use `console.log()` to debug';
      renderMarkdown(markdown);
      
      const codeElement = screen.getByText('console.log()');
      expect(codeElement.tagName.toLowerCase()).toBe('code');
    });
  });

  describe('Blockquotes', () => {
    it('should render blockquotes', () => {
      const markdown = '> This is a quote';
      renderMarkdown(markdown);
      
      const blockquote = document.querySelector('blockquote');
      expect(blockquote).toBeInTheDocument();
      expect(blockquote).toHaveTextContent('This is a quote');
    });
  });

  describe('Tables (GitHub Flavored Markdown)', () => {
    it('should render tables', () => {
      const markdown = `| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |`;
      renderMarkdown(markdown);
      
      const table = screen.getByRole('table');
      expect(table).toBeInTheDocument();
      
      expect(screen.getByText('Header 1')).toBeInTheDocument();
      expect(screen.getByText('Header 2')).toBeInTheDocument();
      expect(screen.getByText('Cell 1')).toBeInTheDocument();
      expect(screen.getByText('Cell 2')).toBeInTheDocument();
    });

    it('should apply correct CSS classes to tables', () => {
      const markdown = `| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |`;
      renderMarkdown(markdown);
      
      const table = screen.getByRole('table');
      expect(table).toHaveClass('border');
      expect(table).toHaveClass('border-slate-300');
      expect(table).toHaveClass('w-full');
    });

    it('should apply correct CSS classes to table headers', () => {
      const markdown = `| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |`;
      renderMarkdown(markdown);
      
      const headers = document.querySelectorAll('th');
      expect(headers.length).toBeGreaterThan(0);
      
      headers.forEach(header => {
        expect(header).toHaveClass('bg-slate-100');
        expect(header).toHaveClass('font-semibold');
        expect(header).toHaveClass('p-3');
        expect(header).toHaveClass('text-left');
        expect(header).toHaveClass('border');
      });
    });

    it('should apply correct CSS classes to table cells', () => {
      const markdown = `| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |`;
      renderMarkdown(markdown);
      
      const cells = document.querySelectorAll('td');
      expect(cells.length).toBeGreaterThan(0);
      
      cells.forEach(cell => {
        expect(cell).toHaveClass('p-3');
        expect(cell).toHaveClass('border');
        expect(cell).toHaveClass('border-slate-200');
      });
    });

    it('should apply correct CSS classes to table rows', () => {
      const markdown = `| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |`;
      renderMarkdown(markdown);
      
      const rows = document.querySelectorAll('tbody tr');
      expect(rows.length).toBeGreaterThan(0);
      
      rows.forEach(row => {
        expect(row).toHaveClass('border-b');
        expect(row).toHaveClass('border-slate-200');
      });
    });
  });

  describe('Complex markdown', () => {
    it('should render complex markdown with multiple features', () => {
      const markdown = `# Title

This is **bold** and *italic* text.

- First item
- Second item

\`\`\`javascript
const code = 'example';
\`\`\`

[Link text](https://example.com)

> This is a quote`;

      renderMarkdown(markdown);
      
      // Check all elements are rendered
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Title');
      expect(screen.getByText('bold')).toBeInTheDocument();
      expect(screen.getByText('italic')).toBeInTheDocument();
      expect(screen.getByRole('list')).toBeInTheDocument();
      expect(screen.getAllByRole('listitem')).toHaveLength(2);
      expect(screen.getByRole('link')).toHaveTextContent('Link text');
      expect(document.querySelector('blockquote')).toBeInTheDocument();
    });

    it('should render GitHub Flavored Markdown features together', () => {
      const markdown = `# Main Title

## Features

- [x] Task list item 1
- [ ] Task list item 2

| Feature | Status |
|---------|--------|
| Markdown | ✅ |
| Tables | ✅ |

\`\`\`rust
fn main() {
    println!("Hello, RustPress!");
}
\`\`\`

> This is a blockquote with **bold** text

[Visit GitHub](https://github.com) for more info.`;

      renderMarkdown(markdown);
      
      // Verify all GFM features are rendered
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Main Title');
      expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent('Features');
      
      // Task list checkboxes
      const checkboxes = document.querySelectorAll('input[type="checkbox"]');
      expect(checkboxes.length).toBeGreaterThanOrEqual(2);
      
      // Table
      const table = screen.getByRole('table');
      expect(table).toBeInTheDocument();
      expect(screen.getByText('Feature')).toBeInTheDocument();
      expect(screen.getByText('Status')).toBeInTheDocument();
      
      // Code block
      const codeBlock = document.querySelector('pre code');
      expect(codeBlock).toBeInTheDocument();
      expect(codeBlock).toHaveTextContent('fn main()');
      
      // Blockquote
      const blockquote = document.querySelector('blockquote');
      expect(blockquote).toBeInTheDocument();
      expect(blockquote).toHaveTextContent('This is a blockquote');
      
      // Link
      const link = screen.getByRole('link');
      expect(link).toHaveTextContent('Visit GitHub');
      expect(link).toHaveAttribute('href', 'https://github.com');
    });
  });

  describe('HTML support', () => {
    it('should render HTML tags when rehype-raw is enabled', () => {
      const markdown = '<div>HTML content</div><strong>Bold HTML</strong>';
      renderMarkdown(markdown);
      
      const div = document.querySelector('div');
      expect(div).toBeInTheDocument();
      expect(div).toHaveTextContent('HTML content');
      
      const strong = document.querySelector('strong');
      expect(strong).toBeInTheDocument();
      expect(strong).toHaveTextContent('Bold HTML');
    });
  });

  describe('List styling', () => {
    it('should apply correct classes to list items', () => {
      const markdown = '- Item 1\n- Item 2';
      renderMarkdown(markdown);
      
      const items = screen.getAllByRole('listitem');
      items.forEach(item => {
        expect(item).toHaveClass('my-1');
      });
    });
  });
});

