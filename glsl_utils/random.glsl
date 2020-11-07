precision highp float;

// if precision is not high resolution, this may fail (e.g. mobile)
float rand(vec2 co) {
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

// this one is better for mobile
// or it's better to make all precision to high
// highp float rand(vec2 co) {
//     highp float a = 12.9898;
//     highp float b = 78.233;
//     highp float c = 43758.5453;
//     highp float dt = dot(co.xy, vec2(a, b));
//     highp float sn = mod(dt, 3.14);
//     return fract(sin(sn) * c);
// }
