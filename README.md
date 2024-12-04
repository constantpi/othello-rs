# othello-rs

othello AI in Rust

## readme

オセロのAIです。
序盤では定石を使い、中盤はモンテカルロ木探索を用いています。
終盤は読み切り(全探索)をします。

## 定石
し定石はsrc/book.rsに書くことにしていますが、著作権の問題で3つしか載せていません。
```
C4:C3
C4C3:D3
C4C5:D6
C4E3:F5
```
のような形式でそこに至るまでの棋譜と打つべき場所を書いてください。

## ビルド方法

```
cargo build
```

でコンパイルできます。

```
cargo run
```

でコンパイルと実行ができます。

## 各ファイルの説明
ai_decide.rs : AIの手を決める関数が書かれています。定石を使うのか、モンテカルロ木探索を使うのか、読み切りをするのかを決めます。
bit_othello.rs : オセロの盤面をビットボードで表現し次に石を置ける場所の計算などをビット演算で行います。
book.rs : 定石を書くファイルです。
command_parser.rs : 通信のプロトコルを実装しています。
depth_first_search.rs : 終盤の読み切りを行います。
main.rs : 実行時の引数を解釈し、ゲームを進行します。サーバーとの通信もここで行います。
monte.rs : モンテカルロ木探索を行います。
proto.rs : 通信のプロトコルを定義しています。
use_book.rs : 定石を使うための関数が書かれています。
