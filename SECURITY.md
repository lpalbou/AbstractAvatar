# Security

Report suspected vulnerabilities privately to contact@abstractframework.ai. Include the affected
version, platform, input and a minimal reproduction. Avoid posting private application data or
an uncoordinated exploit in a public issue.

AbstractAvatar renders trusted built-in procedural designs. Descriptors are bounded parameters,
not scripts or arbitrary plug-ins. Use the bundled native library or WebAssembly artifact from a
trusted build. Do not load native libraries or WebAssembly supplied by an untrusted descriptor.

Hosts remain responsible for validating incoming application messages, controlling frame cadence,
and releasing renderer resources. The native API is a trusted in-process FFI boundary; callers
must obey buffer and synchronization requirements in [the frame contract](docs/frame-contract.md).

See [troubleshooting](docs/troubleshooting.md) for input and resource-limit errors.
