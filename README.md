# 🦀 Rust Strapi-to-Cloudinary Uploader

This Rust application allows you to upload files to [Cloudinary](https://cloudinary.com/) from [Strapi](https://strapi.io/). It works by first sending a GET request to an API endpoint to fetch data about images, and then uploads each image to Cloudinary in chunks asynchronously for increased performance.
## Getting Started
### Prerequisites

- Rust (1.55 or newer)
- An API endpoint that provides a list of images
- A Cloudinary account

## Installation

## Clone the repository:

```bash
git clone https://github.com/user/rust-cloudinary-uploader.git
cd rust-cloudinary-uploader
```

### Install the required dependencies:

```bash
cargo build --release
```

### Environment Variables

The script requires the following environment variables:

```dotenv
CLOUDINARY_URL=<your_cloudinary_url>
API_KEY=<your_api_key>
API_SECRET=<your_api_secret>
BASE_URL=<base_url_of_your_api>
AUTH_TOKEN=<your_api_auth_token>
CHUNK_SIZE=<number_of_images_per_chunk>
FOLDER_NAME=<cloudinary_folder_name>
```

You can create a `.env` file in the root directory and store these values.

### Running the Script

Run the script using:

```bash
cargo run --release
```

## Contributing

If you'd like to contribute, please fork the repository and make changes as you'd like. Pull requests are warmly welcome.
License

This project is licensed under the MIT License - see the LICENSE.md file for details.
