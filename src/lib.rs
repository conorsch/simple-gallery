use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Local directory on-disk where images are stored.
#[derive(Clone)]
pub struct ImageDir {
    /// The filepath of the image directory.
    pub path: PathBuf,
    /// The file extension of image files, for filtering directory contents.
    pub file_extension: String,
}

impl ImageDir {
    /// Walk filesystem directory and return a list of all files matching file extension.
    pub fn find_images(&self) -> Vec<PathBuf> {
        let mut img_files: Vec<PathBuf> = Vec::new();
        for ent in WalkDir::new(self.path.as_os_str()).into_iter().flatten() {
            let path = ent.path();
            if path
                .display()
                .to_string()
                .ends_with(format!(".{}", self.file_extension).as_str())
            {
                img_files.push(path.to_path_buf());
            }
        }
        img_files
    }

    /// Return a single image filepath, as string, at random from the direcotry.
    pub fn get_random_image(&self) -> PathBuf {
        let mut imgs = self.find_images();
        let mut rng = thread_rng();
        imgs.shuffle(&mut rng);
        imgs[0].clone()
    }
}

/// Represents the CSS declarations for fade-in/fade-out transitions
/// between images in the slideshow.
#[derive(Serialize)]
pub struct TransitionConfig {
    pub imgs: Vec<PathBuf>,
    pub title: String,
    pub n_imgs: f32,
    pub duration_per_image: usize,
    pub transition_time: f32,
    // Whether the images should be randomized in order.
    pub shuffle: bool,
    /// The subroute at which the images will be served, e.g. "static/"
    pub static_route: String,
}

impl TransitionConfig {
    /// From a list of images, generator all the animation values for CSS.
    pub fn new(imgs: Vec<PathBuf>, title: String, transition_time: usize, shuffle: bool) -> TransitionConfig {
        // Via https://www.devtwins.com/blog/css-cross-fading-images
        // Define durations in seconds. We use floats so we can do math,
        // and will recast as integers before injecting into CSS.
        let n_imgs: f32 = imgs.len() as f32;
        let duration_per_image: usize = transition_time;
        let transition_time: f32 = 2.0;
        // let animation_delay: f32 = duration_per_image + transition_time;
        let static_route = String::from("static");
        TransitionConfig {
            imgs,
            n_imgs,
            title,
            duration_per_image,
            transition_time,
            static_route,
            shuffle,
        }
    }

    /// Emit full in-line HTML for application. The img-src attributes are the only
    /// external resources loaded.
    pub fn generate_html(&self) -> anyhow::Result<String> {
        // Load the Tera/Jinja template.
        let html_template = include_str!("../files/index.html.j2");
        let mut context = tera::Context::new();
        let imgs: Vec<_> = self
            .imgs
            .iter()
            .map(|i| {
                format!(
                    "{}/{}",
                    self.static_route,
                    i.file_name().unwrap_or_default().to_str().unwrap()
                )
            })
            .collect();
        context.insert("config", &self);
        context.insert("imgs", &imgs);
        context.insert("duration", &self.duration_per_image);
        context.insert("shuffle_opt", &self.shuffle);
        Ok(tera::Tera::one_off(html_template, &context, false)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_math() {
        let mut imgs = Vec::new();
        imgs.push(PathBuf::from("img/foo1.png"));
        imgs.push(PathBuf::from("img/foo2.png"));
        imgs.push(PathBuf::from("img/foo3.png"));
        imgs.push(PathBuf::from("img/foo4.png"));
        let c = TransitionConfig::new(imgs, "foo".to_string(), 5);
        assert!(c.n_imgs == 4.0);
    }

    #[test]
    fn create_image_dir() {
        let d = String::from("img");
        let _i = ImageDir {
            path: d.into(),
            file_extension: String::from("png"),
        };
    }
}
