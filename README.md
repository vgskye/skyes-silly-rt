# Skye's Silly Ray Tracer

It's a silly toy ray tracer, sorta based off of the [Ray Tracing in One Weekend Series](https://raytracing.github.io/).

## Usage
```
skyes-silly-rt [scene] [output]
```
`[scene]` may be a path to a kdl scene description document or one of the two built-in scenes, `builtin:CornellBox` and `builtin:TeapotLand`.
`[output]` is the output path, the file extension must be `.png`.

## Notes
see scene.kdl for an example scene.
the built-in scenes need the STL files in repository root to be in working directory.
custom scenes will also load STL files relative to program working directory, not the scene file!
