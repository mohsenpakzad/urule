<div align="center">
  <img height=150 src="https://github.com/mohsenpakzad/urule/blob/main/src-tauri/icons/icon.png" />
</div>
<p align="center"><strong>Urule</strong></p>

<p align="center"><span>You rule, a memory scanner tool for the Windows operating system.</span></p>

<div align="center">

[![Windows Support](https://img.shields.io/badge/Windows-0078D6?style=flat&logo=windows&logoColor=white)](https://github.com/mohsenpakzad/urule/releases)

</div>

Urule is a tool for scanning, reading and writing memory of processes that are running in our computer.

If you know Cheat Engine, urule is like simplified version of Cheat Engine.

<p float="left">
  <img src="https://github.com/mohsenpakzad/urule/blob/main/screenshots/splash.png" width="49%" />
  <img src="https://github.com/mohsenpakzad/urule/blob/main/screenshots/home.png" width="49%" /> 
</p>

## ✨Features

- Scanning, writing and reading the values of the memories of a process
- Low cpu usage and fast, as the core code is written natively by Rust language
- Low memory usage, as there is no GC usage at core code and UI uses pagination
- Easy to use with simple and elegant UI
- Lightweight size
- Savage logo😎

## 🔍Supported scan types

- Exact value
- Smaller than value
- Bigger than value
- Value between
- Unknown initial value
- Increased value
- Increased value by
- Decreased value
- Decreased value by
- Changed value
- Unchanged value

## 🧬Supported value types

- I8 (Signed Byte)
- U8 (Unsigned Byte)
- I16 (Signed 2 Bytes)
- U16 (Unsigned 2 Bytes)
- I32 (Signed 4 Bytes)
- U32 (Unsigned 4 Bytes)
- I64 (Signed 8 Bytes)
- U64 (Unsigned 8 Bytes)
- F32 (Float 4 Bytes)
- F64 (Float 8 Bytes)

## ⛓Install the dependencies

```bash
pnpm install
```

### Start the app in development mode (hot-code reloading, error reporting, etc.)

```bash
pnpm tauri dev
```

### Build the app for production

```bash
pnpm tauri build
```

## 🔧Troubleshooting

If you face any problem you can see app log file here and send it into support.

```
C:\Users\{YOUR_USER_NAME}\AppData\Roaming\com.github.mohsenpakzad.urule
```

Example:

`C:\Users\Mohsen\AppData\Roaming\com.github.mohsenpakzad.urule`

## 🎉Special thanks
To [@Lonami](https://github.com/Lonami) for his good explanations about [Writing our own Cheat Engine](https://lonami.dev/blog/woce-1/), which helped me a lot to complete this project.



