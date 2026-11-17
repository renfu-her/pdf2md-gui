# pdf2md-gui

Windows 桌面軟體：把 PDF 轉成 Markdown。底層是 [pdf-inspector](https://github.com/firecrawl/pdf-inspector)（Rust 製、純本機解析、免 OCR 即可處理文字型 PDF）。

## 用了什麼

| 東西 | 版本 | 用來做什麼 |
|---|---|---|
| Rust | 1.99.1 | 全部程式語言，單一執行檔 |
| pdf-inspector | 1.25.2 | PDF 分類（文字型／掃描型／圖片型／混合）、文字擷取、轉 Markdown；旗標語義（`--compact`、`--pages` 等）直接對照它的 `pdf2md` 實作 |
| Tauri | 2.12.5 | 桌面殼層（WebView2），前端是 `ui/` 下的純靜態 HTML/CSS/JS、無 npm 建置鏈 |
| rfd | 0.18.0 | 原生檔案選擇／儲存對話框 |
| Inno Setup | 6.7.3 | 打包 Windows 安裝包（`installer/pdf2md-gui.iss`，繁中精靈） |

## 為什麼要做這個界面

`pdf-inspector` 原本只有工程師用的 CLI（`pdf2md`、`detect-pdf`）：參數多（`--select-pages 1,3,5-10`、`--compact`、`--password`…）、輸出是終端機文字，一般使用者沒有辦法用。這個界面把每一個 CLI 旗標變成按鈕與輸入框，上傳一律是「選檔按鈕或拖放」，轉換前畫面會即時組出完全對等的 `pdf2md` 指令並可一鍵複製——會用滑鼠的人就會用，同時保留跟 CLI 一致、可驗證的行為（掃描件一樣是「需要 OCR」而不是空檔）。目標就是：不懂 CLI 的人也能在本機把文字型 PDF（報告、論文、財報、發票）轉成乾淨的 Markdown。

## 功能

- 視窗 512×610 置中、固定不可拉伸，三頁籤：上傳／預覽／關於；主題淺色／深色／自動（跟系統）。
- 上傳：選擇按鈕，或把 PDF 拖進視窗（多檔取第一個，非 PDF 拒收）。
- 選項：緊湊輸出、頁面分隔標記、指定頁面、密碼、僅偵測、版面分析。
- 對等 CLI 指令即時預覽，可複製（密碼顯示為 `****`）。
- 轉換在背景執行緒跑，界面不凍結；結果含偵測報告與 Markdown 預覽，可複製全文或存成 `.md`。
- 主題：淺色／深色／跟隨系統（預設跟隨系統）。

## 上游引用與版權

- 轉換核心是 Firecrawl 的 [`pdf-inspector`](https://github.com/firecrawl/pdf-inspector)（建置版本 1.25.2，見 [Cargo.lock](Cargo.lock)），走 crates.io 依賴引入，**沒有**把上游原始碼放進本 repo。
- CLI 對應做法：界面選項與上游 `pdf2md` 旗標一一對應（`--compact`、`--pages`、`--select-pages`、`--password`、`--detect-only`、`--analyze`），頁碼解析、錯誤訊息與掃描件處理語義都與上游一致；對照關係寫在 `src/convert.rs` 開頭註解。
- 版權：`pdf-inspector` 採用 MIT 授權（Copyright (c) 2026 Firecrawl，LICENSE 隨 crate 發布）。MIT 允許商用、修改與散布，條件是散布物若包含該軟體須附上版權聲明與授權文字。本 repo 目前只以依賴方式使用、未內含上游程式碼；若日後把上游原始碼 vendoring 進來，必須同時放入它的 LICENSE 全文。
- 本專案自有程式碼（GUI 殼層、`ui/`、`installer/`、測試）採用 MIT 授權（Copyright (c) 2026 Patrick Her），全文見 [LICENSE](LICENSE)。

## 開發

```powershell
cargo test            # 單元 + 端到端測試
cargo clippy --all-targets
cargo build --release # 產出 target\release\pdf2md-gui.exe
```

打包安裝包（需 Inno Setup）：

```powershell
& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" installer\pdf2md-gui.iss
# 產出 dist\pdf2md-gui-setup-<版本>.exe（ISCC 路徑依 Inno 實際安裝位置調整）
```

## 版本

目前版本：0.11.0。規則：只要有變更，minor 版號就加 0.1。歷次變更見 [CHANGED.md](CHANGED.md)。

需求：Windows 10/11（內建 WebView2 即可，不需另外安裝）。
