import { Highlight, themes } from 'prism-react-renderer';

interface Props {
  title?: string;
  code: string;
  language?: string;
}

export function CodeBlock({ title, code, language = 'typescript' }: Props) {
  return (
    <div className="rounded-lg overflow-hidden border border-white/10">
      {title && (
        <div className="bg-white/5 px-4 py-2 border-b border-white/10 flex items-center justify-between">
          <span className="text-xs font-mono text-gray-400">{title}</span>
          <span className="text-xs text-gray-500">{language}</span>
        </div>
      )}
      <Highlight theme={themes.nightOwl} code={code.trim()} language={language as any}>
        {({ className, style, tokens, getLineProps, getTokenProps }) => (
          <pre
            className={`${className} p-4 overflow-x-auto text-sm`}
            style={{ ...style, background: 'rgba(0, 0, 0, 0.4)', margin: 0 }}
          >
            {tokens.map((line, i) => (
              <div key={i} {...getLineProps({ line })} className="table-row">
                <span className="table-cell text-gray-600 pr-4 select-none text-right w-8 text-xs">
                  {i + 1}
                </span>
                <span className="table-cell">
                  {line.map((token, key) => (
                    <span key={key} {...getTokenProps({ token })} />
                  ))}
                </span>
              </div>
            ))}
          </pre>
        )}
      </Highlight>
    </div>
  );
}
