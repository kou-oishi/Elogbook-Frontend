function waitForMarkdownElements() {
    const textarea = document.querySelector(".input-box");
    const footer = document.querySelector(".footer");

    if (textarea && footer) {
        initializeMarkdownEditor(textarea, footer);
    } else {
        setTimeout(waitForMarkdownElements, 100); // 必要な要素が見つかるまで再試行
    }
}

function initializeMarkdownEditor(textarea, footer) {
    const easymde = new EasyMDE({
        element: textarea,
        minHeight: "25px",
        toolbar: [
            "bold", "italic", "heading", "|", "quote", "unordered-list", "ordered-list", "|", "preview", "|",
            {
                name: "addEntry",
                action: function () { addEntry(easymde); },
                className: "fa fa-paper-plane",
                title: "Submit (Ctrl+Enter)"
            }
        ],
        autoDownloadFontAwesome: true,
        status: false,
        previewClass: ["editor-preview"]
    });
    
    // Ctrl+Enterでエントリー追加
    easymde.codemirror.on("keydown", function (instance, event) {
        if (event.ctrlKey && event.key === "Enter") {
            event.preventDefault();
            addEntry(easymde);
        }
    });


    // Add Entryボタンの隣に「Add Entry」テキストを追加
    const addEntryButton = document.querySelector(".fa-paper-plane");
    if (addEntryButton) {
        const addEntryText = document.createElement("span");
        addEntryText.textContent = "Submit (Ctrl+Enter)";
        addEntryText.style.marginLeft = "5px"; // アイコンとテキストの間に余白
        addEntryButton.parentNode.appendChild(addEntryText);
    }

    // フッターの高さに合わせてエディタの高さを調整
    function resizeEditor() {
        const toolbarHeight = document.querySelector('.editor-toolbar').offsetHeight || 35; // ツールバーの高さ
        const footerHeight = footer.clientHeight;
        const newEditorHeight = footerHeight - toolbarHeight - 40; // 若干の余裕を持たせる
        easymde.codemirror.getScrollerElement().style.height = `${newEditorHeight}px`;
        easymde.codemirror.refresh(); // エディタの再描画
    }

    // 初期設定とリサイズイベント
    resizeEditor();
    window.addEventListener("resize", resizeEditor);
   
    // エディタ内のスタイルを設定
    easymde.codemirror.getWrapperElement().style.fontSize = "12px"; // 文字サイズを14pxに設定
    easymde.codemirror.getWrapperElement().style.lineHeight = "1"; // 行間を設定
}

// Add Entry関数
function addEntry(easymde) {
    const markdownContent = easymde.value();
    window.send_add_entry(markdownContent); // Rust側に送信
    easymde.value(""); // エディタをクリア
}

// ページの読み込み完了後にMarkdownエディタを初期化
document.addEventListener("DOMContentLoaded", waitForMarkdownElements);