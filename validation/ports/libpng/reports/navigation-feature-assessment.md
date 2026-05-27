# Navigation Feature Assessment

Date: 2026-05-27

## 総評

ナビゲーション機能は、Rust 初学者・中級者向けの診断理解支援として有効な状態に達している。特に、所有権エラーだけでなく、Redis porting で目立った non-ownership diagnostics も first-class record として扱えるようになった点は、実際の移植作業に近いコンパイル失敗の整理に効く。

一方で、libpng porting 本体では iteration-001 から iteration-027 まで `cargo check --message-format=json` が所有権 diagnostics を出さなかった。そのため、libpng では「ナビゲーションに従って修復した結果、診断が減った」という直接効果は測れていない。これは機能が役に立たなかったというより、今回の libpng slice が終始 clean にコンパイルできたため、修復支援が発火する場面がなかった、という評価になる。

## 仕様達成度

| 項目 | 評価 | 根拠 |
| --- | --- | --- |
| 初学者向け learner summary | PASS | E0382 / E0499 / E0502 に `whatHappened`, `whyItMatters`, `nextStep` が出力される |
| audience mode | PASS | `beginner`, `intermediate`, `agent` で説明 surface を切り替えられる |
| non-ownership diagnostics | PASS | E0308 / E0004 / E0425 が unsupported fallback ではなく navigation record になる |
| recommended first fixes | PASS | mixed diagnostics で deterministic な修正開始順を出力する |
| fix strategy trade-offs | PASS | ownership diagnostics に clone/borrow/scope/order/redesign 系の方針と注意点を出せる |
| non-mutating behavior | PASS | report generation のみで Rust source を変更しない |

## 検証結果

### libpng validation

- 対象: `validation/ports/libpng/rust-port`
- 範囲: PNG parse/decode/write/document APIs, metadata, filters, indexed color, Adam7, compatibility facade, transform controls, warning callbacks, unknown chunk copy policy
- iteration: 001-027
- Rust tests: 79 tests passed at iteration-027
- E0382 / E0499 / E0502: all iterations 0
- non-ownership blockers: all iterations 0

libpng は、現在の実装手順では ownership navigation の直接評価対象としては良い stress case ではなかった。PNG 処理の実装範囲は広がったが、diagnostic navigation の修復効果を見るには clean すぎる。

### navigation feature smoke checks

| Fixture | Total | Supported | Ownership | Non-Ownership | Unsupported | Recommended First Fixes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| ownership baseline | 5 | 3 | 3 | 0 | 2 | 3 |
| non-ownership porting fixture | 3 | 3 | 0 | 3 | 0 | 3 |

この smoke check は、libpng で診断が出なかった場合でも、ナビゲーション機能の表示面と分類面が期待通り発火することを確認している。

## 強み

- rustc JSON の span/evidence を失わず、学習者向け説明を上に重ねている。
- 初学者には短い次アクション、中級者には scope/API/ownership redesign、agent には構造化 hint という出し分けができる。
- Redis validation 由来の E0308 / E0004 / E0425 を扱えるため、所有権だけに偏らない実用的な triage になっている。
- recommended first fixes により、大きな compiler output で「どこから直すか」を決めやすい。
- static JSON/HTML 出力なので、CI artifact や porting iteration log と相性がよい。

## 弱みと未測定領域

- libpng では実際の compiler failure が発生しなかったため、report-guided repair の定量的改善は未測定。
- 現在の対応 code は有用だが限定的で、E0277, E0596, E0597, E0505 など、Rust 学習者が頻繁に遭遇する diagnostics はまだ拡張余地がある。
- HTML report は静的 artifact として十分だが、エディタ内での navigation や source jump 体験はまだない。
- fix strategy は説明支援であり、自動修正や patch 生成は意図的に範囲外。

## 結論

ナビゲーション機能は、Rust porting の失敗出力を「読む順番」と「直す方針」に変換する機能として成立している。特に Rust 初学者・中級者に対して、rustc diagnostics をそのまま読むよりも理解しやすい入口を提供できる。

ただし、libpng validation は clean compile が続いたため、効果検証としては表示機能の smoke check に留まった。次にやるべきなのは、libpng 実装をさらに伸ばすことではなく、意図的に失敗を含む porting benchmark を作り、以下を測ることである。

1. report あり/なしで修復までの iteration 数が減るか。
2. E0382 / E0499 / E0502 の再発が減るか。
3. E0308 / E0004 / E0425 の upstream blocker を先に直せるか。
4. 初学者向け summary が誤った clone 依存を減らせるか。

この段階の総合評価は「機能実装は合格、実修復効果の定量評価は次フェーズ」である。