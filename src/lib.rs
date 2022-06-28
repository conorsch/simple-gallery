use serde::Serialize;

/// Store all 
#[derive(Serialize)]
pub struct TransitionConfig {
    pub imgs: Vec<String>,
    pub title: String,
    pub n_imgs: f32,
    pub duration_per_image: f32,
    pub transition_time: f32,
    pub animation_delay: f32,
    pub duration_total: f32,
}

impl TransitionConfig {
    /// From a list of images, generator all the animation values for CSS.
    pub fn new(imgs: Vec<String>, title: String) -> TransitionConfig {
        // Via https://www.devtwins.com/blog/css-cross-fading-images
        // Define durations in seconds. We use floats so we can do math,
        // and will recast as integers before injecting into CSS.
        let n_imgs: f32 = imgs.len() as f32;
        let duration_per_image: f32 = 8.0;
        let transition_time: f32 = 2.0;
        // let animation_delay: f32 = duration_per_image + transition_time;
        let animation_delay: f32 = duration_per_image;
        let duration_total: f32 = animation_delay * n_imgs;
        TransitionConfig {
            imgs,
            n_imgs,
            title,
            duration_per_image,
            transition_time,
            animation_delay,
            duration_total,
        }
    }
    /// Return a vec of tuples, for declaring the keyframes of the CSS transition
    /// animation. Tuples are e.g. (0, 1) where 0 is 0% of progress through
    /// the animation, and 1 is "opacity: 1".
    pub fn keyframes(&self) -> Vec<(i32, i32)> {
        // Define keyframe intervals, so the transition between images is smooth
        let keyframe_1: f32 = 0.0;
        let keyframe_2: f32 = (self.duration_per_image / self.duration_total) * 100.0;
        let keyframe_3: f32 =
            ((self.duration_per_image + self.transition_time) / self.duration_total) * 100.0;
        let keyframe_4: f32 = 100.0 - ((self.transition_time / self.duration_total) * 100.0);
        let keyframe_5: f32 = 100.0;
        // The keys are the computed keyframes; the values are hardcoded,
        // describing the opacity gradient for fadein/fadeout.
        Vec::from([
            (keyframe_1.round() as i32, 0),
            (keyframe_2.round() as i32, 1),
            (keyframe_3.round() as i32, 0),
            (keyframe_4.round() as i32, 0),
            (keyframe_5.round() as i32, 1),
        ])
    }

    /// Emit full in-line HTML for application. The img-src attributes are the only
    /// external resources loaded.
    pub fn generate_html(&self) -> String {
        // Load the Tera/Jinja template.
        let html_template = include_str!("../files/index.html.j2");
        let mut context = tera::Context::new();
        context.insert("config", &self);
        context.insert("keyframes", &self.keyframes());

        let result = tera::Tera::one_off(html_template, &context, false);
        result.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_math() {
        let mut imgs = Vec::new();
        imgs.push("img/foo1.png".to_owned());
        imgs.push("img/foo2.png".to_owned());
        imgs.push("img/foo3.png".to_owned());
        imgs.push("img/foo4.png".to_owned());
        let c = TransitionConfig::new(imgs, "foo".to_string());
        assert!(c.n_imgs == 4.0);
        assert!(c.animation_delay == 8.0);
        assert!(c.duration_total == 32.0);
        for (k, v) in c.keyframes() {
            assert!(k >= 0);
            assert!(k <= 100);
            assert!(v == 0 || v == 1);
        }
    }
}
