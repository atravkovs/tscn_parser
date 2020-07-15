use crate::types::Curve;

use nalgebra::Vector2;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum VarType {
    Num(isize),
    Bool(bool),
    Float(f32),
    Str(String),
    Curve(Curve),
    Rect2([Vector2<f32>; 2]),
    IntArr(Vec<isize>),
    FloatArr(Vec<f32>),
    Vector(Vector2<f32>),
    VectorArr(Vec<Vector2<f32>>),
    ArrMap(Vec<HashMap<String, VarType>>),
    Map(HashMap<String, VarType>),
    SubResource(usize),
    ExtResource(usize),
    None(String),
}

impl TryFrom<VarType> for isize {
    type Error = ();

    fn try_from(value: VarType) -> Result<Self, Self::Error> {
        if let VarType::Num(number) = value {
            Ok(number)
        } else {
            Err(())
        }
    }
}

impl TryFrom<VarType> for f32 {
    type Error = ();

    fn try_from(value: VarType) -> Result<Self, Self::Error> {
        if let VarType::Float(number) = value {
            Ok(number)
        } else {
            Err(())
        }
    }
}

impl TryFrom<VarType> for bool {
    type Error = ();

    fn try_from(value: VarType) -> Result<Self, Self::Error> {
        if let VarType::Bool(boolean) = value {
            Ok(boolean)
        } else {
            Err(())
        }
    }
}

impl TryFrom<VarType> for String {
    type Error = ();

    fn try_from(value: VarType) -> Result<Self, Self::Error> {
        if let VarType::Str(string) = value {
            Ok(string)
        } else {
            Err(())
        }
    }
}

impl TryFrom<VarType> for Curve {
    type Error = ();

    fn try_from(value: VarType) -> Result<Self, Self::Error> {
        if let VarType::Curve(curve) = value {
            Ok(curve)
        } else {
            Err(())
        }
    }
}

impl TryFrom<&VarType> for isize {
    type Error = ();

    fn try_from(value: &VarType) -> Result<Self, Self::Error> {
        if let VarType::Num(number) = value {
            Ok(*number)
        } else {
            Err(())
        }
    }
}

impl TryFrom<&VarType> for f32 {
    type Error = ();

    fn try_from(value: &VarType) -> Result<Self, Self::Error> {
        if let VarType::Float(number) = value {
            Ok(*number)
        } else {
            Err(())
        }
    }
}

impl TryFrom<&VarType> for bool {
    type Error = ();

    fn try_from(value: &VarType) -> Result<Self, Self::Error> {
        if let VarType::Bool(boolean) = value {
            Ok(*boolean)
        } else {
            Err(())
        }
    }
}
