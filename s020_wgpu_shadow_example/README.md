
## Memo

- 2つのレンダーパスで描画する
  - `shadow_pass`
    - 影のテクスチャを各光源ごとに `shadow_texture` に書き込む
    - フラグメントステージはなし
  - `forward_pass`
    - `shadow_texture` を `shadow_texture_view` を介して共有
    - `shadow_texture_view` は `uniform` としてバインドされているので `forward.frag` で読める
- 基本的な設定の流れは以下
  - 初期化処理
    - `vertex_buffer` `index_buffer` `texture` `sampler` を準備
    - `uniform buffer` 準備
    - `bind_group_layout` にバインドする `uniform` のメタデータを設定
    - `bind_group` に上記設定に基づいて実際に `uniform_buffer` をバインド
    - `pipeline_layout` に必要な `bind_group_layout` をアタッチ
    - `pipeline` を各ステージに必要な情報を与えて作成
  - ループ処理
    - `uniform_buffer` を更新
    - `uniform_buffer` を `queue` に積む
    - `command_encoder` で `render_pass` を作成
    - `color_attatchments` に出力するカラーテクスチャを指定
    - `depth_stencil_attachment` に出力する深度・ステンシルバッファを指定
    - `pipeline` を `render_pass` にセットする
    - `bind_group` (`uniform_buffer`) を `render_pass` にセットする
    - `vertex_buffer` `index_buffer` `texture` `sampler` 等を `render_pass` にセットする
    - `draw_indexed` でテクスチャへ描画する
    - `raw_frame` の場合は `raw_frame.swap_chain_texture()` で出力先の `texture_view` を取得できる
- projection matrix などの共通 `uniform` は `encoder_copy_buffer_to_buffer` でコピーして使用
- `raw_frame()` を使わないと nannou のデフォルトの描画設定が使われてしまうので注意


## Reference

- https://github.com/gfx-rs/wgpu-rs/tree/master/examples/shadow
- [3DグラフィクスAPI Vulkan を出来るだけやさしく解説する本](https://techbookfest.org/product/5078992340123648?productVariantID=6740057192923136) が非常に分かりやすかった
