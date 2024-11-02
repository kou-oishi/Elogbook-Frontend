function waitForElements() {
    const content = document.querySelector(".content");
    const footer = document.querySelector(".footer");
    const divider = document.querySelector(".resize-divider");
    const textarea = document.querySelector(".input-box");

    if (content && footer && divider && textarea) {
        initializeResizing(content, footer, divider, textarea);
    } else {
        // 必要な要素が見つかるまで100ms間隔で再試行
        setTimeout(waitForElements, 100);
    }
}

function initializeResizing(content, footer, divider, textarea) {
    let isResizing = false;

    divider.addEventListener("mousedown", function (e) {
        isResizing = true;
        document.body.style.cursor = "ns-resize";
    });

    document.addEventListener("mousemove", function (e) {
        if (!isResizing) return;

        const newHeight = window.innerHeight - e.clientY;
        content.style.height = `${window.innerHeight - newHeight}px`;
        footer.style.height = `${newHeight}px`;

        // テキストエリアの高さをfooterに合わせて変更
        textarea.style.height = `${newHeight - 20}px`; // paddingを考慮
    });

    document.addEventListener("mouseup", function () {
        isResizing = false;
        document.body.style.cursor = "default";
    });
}

// 初期化関数を呼び出して要素が見つかるまで待機
document.addEventListener("DOMContentLoaded", waitForElements);
