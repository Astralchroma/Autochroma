use crate::{modules::Module, Context, Data, Error, GenericError, Result};
use image::imageops::{overlay, resize, FilterType};
use image::{io::Reader, DynamicImage, ImageOutputFormat, Rgba, RgbaImage};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use log::error;
use once_cell::sync::Lazy;
use poise::{command, serenity_prelude::Attachment, serenity_prelude::AttachmentType, Command};
use std::io::Cursor;

// We don't actually know who owns the blobfox emojis, so we can't include the source image in the project.
// So we load it from a file expected to be located at `./blobfoxcute.png` and `./blobfoxcute_snout.png`.
// If you don't have these files, the nom module will not be usable.
// At some point we should look into improving this.

// Expected to be a 120px :blobfoxcute:
static BLOBFOX: Lazy<Option<DynamicImage>> = Lazy::new(|| load_image("blobfoxcute.png"));

// Expected to be a 120px :blobfoxcute: snout overlay, rest of the image should be transparent
static BLOBFOX_SNOUT: Lazy<Option<DynamicImage>> = Lazy::new(|| load_image("blobfoxcute_snout.png"));

fn load_image(name: &'static str) -> Option<DynamicImage> {
	match Reader::open(name) {
		Ok(image) => match image.decode() {
			Ok(image) => Some(image),
			Err(error) => {
				error!("Failed to load {name}: {error}");
				None
			}
		},
		Err(error) => {
			error!("Failed to load {name}: {error}");
			None
		}
	}
}

pub struct Nom;

impl Nom {
	#[command(slash_command)]
	async fn nom(context: Context<'_>, image: Attachment) -> Result<()> {
		let mut output = RgbaImage::new(128, 128);

		{
			let blobfox = BLOBFOX.as_ref().ok_or(GenericError("BLOBFOX image failed to load"))?;
			overlay(&mut output, blobfox, 4, 0);
		}

		{
			let image_bytes = image.download().await?;
			let image = Reader::new(Cursor::new(image_bytes))
				.with_guessed_format()?
				.decode()?
				.into_rgba8();

			let image = resize(&image, 64, 64, FilterType::Nearest);

			let mut working_image = RgbaImage::new(128, 128);
			overlay(&mut working_image, &image, 32, 32);

			let working_image = rotate_about_center(
				&working_image,
				f32::to_radians(-90.0 + 25.0),
				Interpolation::Bilinear,
				Rgba([0, 0, 0, 0]),
			);

			overlay(&mut output, &working_image, 12 - 32, 64 - 32);
		}

		{
			let snout = BLOBFOX_SNOUT
				.as_ref()
				.ok_or(GenericError("BLOBFOX_SNOUT image failed to load"))?;
			overlay(&mut output, snout, 4, 0);
		}

		let mut output_bytes = vec![];
		output.write_to(&mut Cursor::new(&mut output_bytes), ImageOutputFormat::Png)?;

		context
			.send(|reply| {
				reply.attachment(AttachmentType::Bytes {
					data: output_bytes.into(),
					filename: String::from("blobfoxnom.png"),
				})
			})
			.await?;

		Ok(())
	}
}

impl Module for Nom {
	const NAME: &'static str = "nom";

	fn append_commands(commands: &mut Vec<Command<Data, Error>>) {
		commands.push(Self::nom());
	}
}
