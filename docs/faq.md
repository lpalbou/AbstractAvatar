# Frequently asked questions

## Does each platform need its own procedural interpreter?

No. Native hosts call the compiled library; browsers run the same generator compiled to
WebAssembly. Platform presenters display the frames and own the clock, palette and input state.
See [architecture](architecture.md).

## Is this a standard animation-file format?

The request descriptor is a small AbstractAvatar parameter contract. Frames use RGBA pixels and
PNG previews; WebAssembly supplies portable execution. It is not an arbitrary animation-file
loader. See [the frame contract](frame-contract.md).

## Does rendering need an AI model, microphone or server?

No. Your application supplies activity and optional audio levels. Generation is local and does
not capture audio or make network requests.

## Must a general-purpose TUI depend on avatars?

No. A generic frame player can present application-supplied frames independently. Applications
opt into an avatar source when they want one.

## Are all source clients automatically migrated?

No. The shared catalog and generator are reusable independently of client adoption. Watch and
iPhone implementations remain separate until explicitly integrated. See [catalog](catalog.md).
