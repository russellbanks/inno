#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompressionFilter {
    #[default]
    NoFilter,
    InstructionFilter4108,
    InstructionFilter5200,
    InstructionFilter5309,
    ZlibFilter,
}
