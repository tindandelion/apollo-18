use crate::color::LinearRgb;

pub(crate) trait FragmentShader {
    type Attribute: Copy;

    fn shade(&self, attributes: [Self::Attribute; 3], barycentric_weights: [f32; 3]) -> LinearRgb;
}
