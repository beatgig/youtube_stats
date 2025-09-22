# Youtube Stats

A Rust library with Python bindings for Youtube stats

## Installation

```bash
pip install youtube_stats
```


# Local Development

## Clone the repo

```bash
git clone git@github.com:beatgig/youtube_stats.git
cd youtube_stats
```

## Configuration

Create a .env file in the root of the project with the following contents:

```bash
YOUTUBE_API_KEY=YOUR_API_KEY
```

## Build Locally and Install the Extension in Python Environment

```bash
poetry run maturin develop
```

## Run Tests

```bash
pytest -s test/
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

## Deployment

Any push to `main` will trigger a GitHub Actions workflow that will build and publish the Python package to PyPI.

## License

MIT
