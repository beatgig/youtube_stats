# Youtube Stats

A Rust library with Python bindings for Youtube stats

## Installation

```bash
pip install youtube_stats
```

## Configuration

Create a .env file in the root of the project with the following contents:

```bash
YOUTUBE_API_KEY=YOUR_API_KEY
```

## Usage

### Python

```python
from youtube_stats import YoutubeStats

stats = YoutubeStats()
stats.get_stats("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
```

### Rust

```rust
use youtube_stats::YoutubeStats;

let stats = YoutubeStats::new();
stats.get_stats("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
```

## License

MIT
