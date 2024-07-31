use anyhow::Result;
use rusty_ytdl::{FFmpegArgs, RequestOptions, Video, VideoOptions};
use std::fs::create_dir_all;
use std::io::Write;
use std::path::PathBuf;

const MEDIAS_FOLDER: &str = "medias";
const INVALID_CHARS: [char; 1] = ['/'];

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Youtube url
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let url = &cli.url;
    println!("url {}", url);

    let media_folder = PathBuf::from(MEDIAS_FOLDER);
    create_dir_all(&media_folder).expect("Unable to create medias folder");
    fetch_to_mp3(url, &media_folder).await?;
    println!("Downloading success.");

    Ok(())
}

async fn fetch_to_mp3(url: &str, media_folder: &PathBuf) -> Result<()> {
    let video_options = VideoOptions {
        request_options: RequestOptions {
            cookies: None,
            ..Default::default()
        },
        ..Default::default()
    };
    let video = Video::new_with_options(url, video_options).expect(&format!(
        "Cannot get video id with provided url or id: {}",
        url
    ));
    let video_info = video.get_basic_info().await?;
    let video_title = video_info
        .video_details
        .title
        .trim()
        .chars()
        .map(|ch| if INVALID_CHARS.contains(&ch) { '_' } else { ch })
        .collect::<String>();
    let path = media_folder.join(&video_title).with_extension("mp3");
    println!("Output path: {:?}", path);
    println!("Video title: {}", video_title);
    println!("Fetch the audio stream...");
    let stream = video
        .stream_with_ffmpeg(Some(FFmpegArgs {
            format: Some(String::from("mp3")),
            audio_filter: None,
            video_filter: None,
        }))
        .await?;

    println!("Start to generating mp3 file...");
    let mut file = std::fs::File::create(path)?;
    while let Some(chunk) = stream.chunk().await? {
        file.write_all(&chunk)?;
    }

    Ok(())
}
