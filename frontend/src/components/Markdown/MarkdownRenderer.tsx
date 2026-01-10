import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeHighlight from 'rehype-highlight';
import { normalizeImageUrl } from '../../utils/url';
import 'highlight.js/styles/github.css';
import './markdown.css';

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

export function MarkdownRenderer({ content, className = '' }: MarkdownRendererProps) {
  return (
    <div className={`markdown-body ${className}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeRaw, rehypeHighlight]}
        components={{
          // Headings with anchor links (GitHub-style)
          h1: ({node, className, children, ...props}: any) => (
            <h1 className={`${className || ''} markdown-heading markdown-h1`} {...props}>
              {children}
            </h1>
          ),
          h2: ({node, className, children, ...props}: any) => (
            <h2 className={`${className || ''} markdown-heading markdown-h2`} {...props}>
              {children}
            </h2>
          ),
          h3: ({node, className, children, ...props}: any) => (
            <h3 className={`${className || ''} markdown-heading markdown-h3`} {...props}>
              {children}
            </h3>
          ),
          h4: ({node, className, children, ...props}: any) => (
            <h4 className={`${className || ''} markdown-heading markdown-h4`} {...props}>
              {children}
            </h4>
          ),
          h5: ({node, className, children, ...props}: any) => (
            <h5 className={`${className || ''} markdown-heading markdown-h5`} {...props}>
              {children}
            </h5>
          ),
          h6: ({node, className, children, ...props}: any) => (
            <h6 className={`${className || ''} markdown-heading markdown-h6`} {...props}>
              {children}
            </h6>
          ),
          
          // Paragraphs
          p: ({node, className, children, ...props}: any) => (
            <p className={`${className || ''} markdown-paragraph`} {...props}>
              {children}
            </p>
          ),
          
          // Links (GitHub-style blue)
          a: ({node, className, href, children, ...props}: any) => (
            <a
              href={href}
              className={`${className || ''} markdown-link`}
              target={href?.startsWith('http') ? '_blank' : undefined}
              rel={href?.startsWith('http') ? 'noopener noreferrer' : undefined}
              {...props}
            >
              {children}
            </a>
          ),
          
          // Images
          img: ({node, className, src, alt, ...props}: any) => {
            const normalizedSrc = normalizeImageUrl(src || '');
            return (
              <img
                src={normalizedSrc}
                alt={alt}
                className={`${className || ''} markdown-image`}
                {...props}
              />
            );
          },
          
          // Code blocks and inline code
          code: ({node, inline, className, children, ...props}: any) => {
            if (inline) {
              return (
                <code className={`${className || ''} markdown-code-inline`} {...props}>
                  {children}
                </code>
              );
            }
            return (
              <code className={`${className || ''} markdown-code-block`} {...props}>
                {children}
              </code>
            );
          },
          pre: ({node, className, children, ...props}: any) => (
            <pre className={`${className || ''} markdown-pre`} {...props}>
              {children}
            </pre>
          ),
          
          // Lists
          ul: ({node, className, children, ...props}: any) => (
            <ul className={`${className || ''} markdown-list markdown-list-unordered`} {...props}>
              {children}
            </ul>
          ),
          ol: ({node, className, children, ...props}: any) => (
            <ol className={`${className || ''} markdown-list markdown-list-ordered`} {...props}>
              {children}
            </ol>
          ),
          li: ({node, className, children, ...props}: any) => (
            <li className={`${className || ''} markdown-list-item`} {...props}>
              {children}
            </li>
          ),
          
          // Blockquotes (GitHub-style with left border)
          blockquote: ({node, className, children, ...props}: any) => (
            <blockquote className={`${className || ''} markdown-blockquote`} {...props}>
              {children}
            </blockquote>
          ),
          
          // Tables (GitHub-style)
          table: ({node, className, children, ...props}: any) => (
            <div className="markdown-table-wrapper">
              <table className={`${className || ''} markdown-table`} {...props}>
                {children}
              </table>
            </div>
          ),
          thead: ({node, className, children, ...props}: any) => (
            <thead className={`${className || ''} markdown-table-head`} {...props}>
              {children}
            </thead>
          ),
          tbody: ({node, className, children, ...props}: any) => (
            <tbody className={`${className || ''} markdown-table-body`} {...props}>
              {children}
            </tbody>
          ),
          tr: ({node, className, children, ...props}: any) => (
            <tr className={`${className || ''} markdown-table-row`} {...props}>
              {children}
            </tr>
          ),
          th: ({node, className, children, ...props}: any) => (
            <th className={`${className || ''} markdown-table-header`} {...props}>
              {children}
            </th>
          ),
          td: ({node, className, children, ...props}: any) => (
            <td className={`${className || ''} markdown-table-cell`} {...props}>
              {children}
            </td>
          ),
          
          // Horizontal rule
          hr: ({node, className, ...props}: any) => (
            <hr className={`${className || ''} markdown-hr`} {...props} />
          ),
          
          // Text formatting
          strong: ({node, className, children, ...props}: any) => (
            <strong className={`${className || ''} markdown-strong`} {...props}>
              {children}
            </strong>
          ),
          em: ({node, className, children, ...props}: any) => (
            <em className={`${className || ''} markdown-em`} {...props}>
              {children}
            </em>
          ),
          del: ({node, className, children, ...props}: any) => (
            <del className={`${className || ''} markdown-del`} {...props}>
              {children}
            </del>
          ),
          
          // Task lists (GitHub Flavored Markdown)
          input: ({node, className, type, checked, ...props}: any) => {
            if (type === 'checkbox') {
              return (
                <input
                  type="checkbox"
                  className={`${className || ''} markdown-checkbox`}
                  checked={checked}
                  disabled
                  readOnly
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
}
