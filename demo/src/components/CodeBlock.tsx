interface Props {
  title?: string;
  code: string;
  language?: string;
}

export function CodeBlock({ title, code, language = 'typescript' }: Props) {
  return (
    <div className="rounded-lg overflow-hidden border border-white/10">
      {title && (
        <div className="bg-white/5 px-4 py-2 border-b border-white/10">
          <span className="text-xs font-mono text-gray-400">{title}</span>
        </div>
      )}
      <pre className={`p-4 bg-black/40 overflow-x-auto text-sm language-${language}`}>
        <code className="text-gray-300">{code}</code>
      </pre>
    </div>
  );
}

