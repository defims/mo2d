//! B7: JSON Schema emission for the v1 envelope (file-format.md rule 6 —
//! the spec comes first; the engine's runtime validator in `validate.rs` is
//! the enforcement, this schema is its machine-readable twin).

/// The v1 JSON Schema (draft-07 shape, deterministic text). It mirrors what
/// `format.rs` writes and `validate.rs` enforces — including the Phase B
/// vocabulary (styles / brush / pressure / uv / texture / the seven
/// relation types).
pub fn schema_v1() -> &'static str {
    r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "mo2d document",
  "description": "mo2d v1 envelope: everything is a line; relations are functions. Byte-level determinism: writers emit canonical field order; unknown keys pass through untouched.",
  "type": "object",
  "required": ["format", "version"],
  "properties": {
    "format": {"const": "mo2d"},
    "version": {"type": "integer", "minimum": 1, "maximum": 1},
    "minReader": {"type": "integer", "minimum": 1},
    "generator": {"type": "string"},
    "root": {
      "type": "object",
      "properties": {
        "width": {"type": "integer", "minimum": 1, "maximum": 16384},
        "height": {"type": "integer", "minimum": 1, "maximum": 16384},
        "ticksPerSecond": {"const": 254016000000},
        "fps": {"type": "number", "exclusiveMinimum": 0, "maximum": 1000}
      }
    },
    "lines": {"type": "array", "items": {"$ref": "#/definitions/line"}},
    "relations": {"type": "array", "items": {"$ref": "#/definitions/relation"}},
    "styles": {"type": "array", "items": {"$ref": "#/definitions/style"}}
  },
  "definitions": {
    "color": {"type": "array", "items": {"type": "integer", "minimum": 0, "maximum": 255}, "minItems": 4, "maxItems": 4},
    "vec3": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3},
    "uv": {"type": "array", "items": {"type": "number", "minimum": 0, "maximum": 1}, "minItems": 2, "maxItems": 2},
    "line": {
      "type": "object",
      "required": ["id", "nodes"],
      "properties": {
        "id": {"type": "string", "minLength": 1},
        "tags": {"type": "array", "items": {"type": "string"}},
        "closed": {"type": "boolean"},
        "width": {"type": "number", "minimum": 0},
        "color": {"$ref": "#/definitions/color"},
        "fillColor": {"$ref": "#/definitions/color"},
        "strokeOn": {"type": "boolean"},
        "fillOn": {"type": "boolean"},
        "brush": {"$ref": "#/definitions/brush"},
        "texture": {"type": "string", "description": "external image ref (B5); resolved against the blob area, then the document directory"},
        "nodes": {"type": "array", "items": {"$ref": "#/definitions/node"}},
        "segs": {"type": "array", "items": {"$ref": "#/definitions/seg"}}
      }
    },
    "node": {
      "type": "object",
      "required": ["p"],
      "properties": {
        "p": {"oneOf": [{"$ref": "#/definitions/vec3"}, {"type": "array", "items": {"type": "integer"}, "minItems": 1, "maxItems": 1, "description": "track tick"}]},
        "w": {"type": "number", "minimum": 0},
        "pr": {"type": "number", "minimum": 0, "maximum": 10, "description": "pressure (B2)"},
        "uv": {"$ref": "#/definitions/uv"},
        "c": {"$ref": "#/definitions/color"},
        "hi": {"$ref": "#/definitions/vec3"},
        "ho": {"$ref": "#/definitions/vec3"},
        "v": {"description": "track keyframe value: number or array"}
      }
    },
    "seg": {
      "type": "object",
      "properties": {
        "type": {"enum": ["straight", "curve"]},
        "visible": {"type": "boolean"},
        "interp": {
          "type": "object",
          "properties": {
            "im": {"type": "integer", "enum": [0, 9], "description": "0=Linear, 9=Bezier"},
            "b": {"type": "array", "items": {"type": "object", "properties": {
              "ao": {"type": "number"}, "ai": {"type": "number"},
              "po": {"type": "number", "minimum": 0, "maximum": 1},
              "pi": {"type": "number", "minimum": 0, "maximum": 1}
            }}}
          }
        }
      }
    },
    "brush": {
      "type": "object",
      "description": "B2 15-parameter brush family; engine consumes sizeAmp/sizeScale/tint deterministically, the rest is stored payload",
      "properties": {
        "name": {"type": "string"},
        "align": {"type": "number"}, "jitter": {"type": "number"},
        "spacing": {"type": "number", "minimum": 0, "maximum": 1},
        "angleDrift": {"type": "number"}, "randomize": {"type": "boolean"},
        "mergedAlpha": {"type": "boolean"}, "tint": {"$ref": "#/definitions/color"},
        "randOrder": {"type": "boolean"},
        "sizeAmp": {"type": "number", "minimum": 0}, "sizeScale": {"type": "number", "minimum": 0},
        "randomInterval": {"type": "number"},
        "hueDrift": {"type": "number"}, "satDrift": {"type": "number"}, "valDrift": {"type": "number"}
      }
    },
    "style": {
      "type": "object",
      "required": ["id"],
      "description": "B2 named line-attribute template",
      "properties": {
        "id": {"type": "string"},
        "width": {"type": "number"}, "color": {"$ref": "#/definitions/color"},
        "fillColor": {"$ref": "#/definitions/color"},
        "strokeOn": {"type": "boolean"}, "fillOn": {"type": "boolean"},
        "brush": {"$ref": "#/definitions/brush"}
      }
    },
    "relation": {
      "type": "object",
      "required": ["id", "type"],
      "oneOf": [
        {"properties": {"type": {"const": "bind"}, "target": {"type": "string"}, "skeleton": {"type": "string"},
          "falloff": {"type": "number", "minimum": 0}, "stretch": {"type": "boolean"}}},
        {"properties": {"type": {"const": "share"}, "driver": {"$ref": "#/definitions/nodeRef"}, "driven": {"$ref": "#/definitions/nodeRef"}}},
        {"properties": {"type": {"const": "track"}, "trackLine": {"type": "string"},
          "target": {"type": "object", "properties": {"line": {"type": "string"}, "node": {"type": "integer", "minimum": 0},
            "attr": {"enum": ["position", "width", "color"]}}}}},
        {"properties": {"type": {"const": "gradient"}, "bar": {"type": "string"}, "target": {"type": "string"},
          "role": {"enum": ["fill", "stroke"]}}},
        {"properties": {"type": {"const": "group"}, "box": {"type": "string"},
          "members": {"type": "array", "items": {"type": "string"}, "minItems": 1}, "visible": {"type": "boolean"}}},
        {"properties": {"type": {"const": "sim"}, "line": {"type": "string"},
          "anchors": {"type": "array", "items": {"type": "object", "properties": {
            "node": {"type": "integer", "minimum": 0}, "driver": {"$ref": "#/definitions/nodeRef"}}}},
          "gravity": {"$ref": "#/definitions/vec3"}, "damping": {"type": "number", "minimum": 0, "maximum": 1},
          "iterations": {"type": "integer", "minimum": 0, "maximum": 100}}},
        {"properties": {"type": {"const": "particles"}, "emitter": {"type": "string"}, "node": {"type": "integer", "minimum": 0},
          "rate": {"type": "number", "exclusiveMinimum": 0}, "speed": {"type": "number", "minimum": 0},
          "dir": {"$ref": "#/definitions/vec3"}, "spread": {"type": "number", "minimum": 0, "maximum": 360},
          "life": {"type": "number", "exclusiveMinimum": 0}, "gravity": {"$ref": "#/definitions/vec3"},
          "damping": {"type": "number", "minimum": 0, "maximum": 1}, "size": {"type": "number", "minimum": 0},
          "color": {"$ref": "#/definitions/color"}, "len": {"type": "number", "minimum": 0},
          "seed": {"type": "integer", "description": "explicit deterministic seed"}}}
      ]
    },
    "nodeRef": {
      "type": "object",
      "properties": {"line": {"type": "string"}, "node": {"type": "integer", "minimum": 0}}
    }
  }
}
"##
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_text_is_deterministic() {
        assert_eq!(super::schema_v1(), super::schema_v1());
        assert!(super::schema_v1().contains("\"const\": 254016000000"));
        assert!(super::schema_v1().contains("\"seed\""));
    }
}
