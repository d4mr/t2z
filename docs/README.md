# t2z Documentation

Documentation for t2z - a library for building Zcash transactions from transparent inputs to shielded Orchard outputs.

**Live documentation: [t2z.d4mr.com](https://t2z.d4mr.com)**

## Local Development

Install the [Mintlify CLI](https://www.npmjs.com/package/mint):

```bash
npm i -g mint
```

Run the development server:

```bash
cd docs
mint dev
```

View at `http://localhost:3000`.

## Structure

```
docs/
├── index.mdx           # Home page
├── quickstart.mdx      # Getting started guide
├── concepts.mdx        # Core concepts
├── flow/               # Transaction flow guides
├── platforms/          # Platform-specific guides (TS, Go, Kotlin)
├── guides/             # Advanced guides
├── api-reference/      # API documentation
└── docs.json           # Mintlify configuration
```

## Deployment

Changes pushed to the main branch are automatically deployed via Mintlify's GitHub integration.
