
## Memo

- 法線マップは (0.0, 1.0) の範囲で法線を定義しているので、(-1.0, 1.0) にマップする必要がある
- 法線・接線・従接線が必要になる (normal, tangent, bitangent)
- 上記の情報から法線ベクトルを接空間へ変換して計算を行う
- 法線マップは通常のテクスチャと同じ `Rgba8UnormSrgb` では[精度が落ちる](https://medium.com/@bgolus/generating-perfect-normal-maps-for-unity-f929e673fc57#b86c)ので `Rgba8Unorm` を使う


## Reference

- https://sotrh.github.io/learn-wgpu/intermediate/tutorial10-lighting/
- http://www.opengl-tutorial.org/jp/intermediate-tutorials/tutorial-13-normal-mapping/
- [View Space で Phong を計算するサンプル](https://learnopengl.com/code_viewer_gh.php?code=src/2.lighting/2.4.basic_lighting_exercise2/basic_lighting_exercise2.cpp)
- [3DグラフィクスAPI Vulkan を出来るだけやさしく解説する本](https://techbookfest.org/product/5078992340123648?productVariantID=6740057192923136) が非常に分かりやすかった
