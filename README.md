# rustychroma
A high-performance key background removal and edge erosion for Rust, C, C++, and WebAssembly. It is a port of [chromakey](https://github.com/t7ru/chromakey).

## Usage
Rust:
```rust
use rustychroma::remove;

fn main() {
	// 1080p RGBA buffer [1]
	let mut pixels = vec![0u8; 1920 * 1080 * 4]; 
	
	// Remove green with a threshold of 7000.0 [2]
	remove(&mut pixels, 0, 255, 0, 7000.0);
}
```

C:
```c
#include "rustychroma.h"

int main() {
	// [1]
	uintptr_t len = 1920 * 1080 * 4;
	uint8_t* pixels = malloc(len);
	
	// [2]
	chromakey_remove(pixels, len, 0, 255, 0, 7000.0);
	
	free(pixels);
	return 0;
}
```

WebAssembly:
```js
import init, { remove } from './dist/web/rustychroma.js';

async function run() {
	await init();
	
	// [1]
	const pixels = new Uint8Array(1920 * 1080 * 4);
	// [2]
	remove(pixels, 0, 255, 0, 7000.0);
}
```

## Functions
- **remove()**: Removes pixels within the given BT.601 chroma distance of the key color. Optional multi-threading via `parallel`.
- **remove_range()**: Soft chroma key using BT.601 chroma distance. Pixels within `min_threshold` become fully transparent, pixels beyond `max_threshold` are kept, and pixels in between receive proportional transparency and color spill suppression.
- **erode()**: Removes exactly 1 pixel of alpha along all edges by clearing any opaque pixel adjacent to a fully transparent pixel.
