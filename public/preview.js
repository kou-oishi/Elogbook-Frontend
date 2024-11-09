window.addEventListener("DOMContentLoaded", waitForContentElement);

function waitForContentElement() {
  if (window.content) {
    initializeAttachmentPreviews();
    observeContentChanges();
  } else {
    setTimeout(waitForContentElement, 100);
  }
}

function observeContentChanges() {
  const observer = new MutationObserver(() => {
    initializeAttachmentPreviews();
  });
  const config = { childList: true, subtree: true };
  observer.observe(window.content, config);
}

function initializeAttachmentPreviews() {
  const attachmentElements = window.content.querySelectorAll(".image-attachment, .text-attachment, .pdf-attachment");

  attachmentElements.forEach(element => {
    if (!element.dataset.initialized) {
      const url = element.getAttribute("data-url");
      const id = element.getAttribute("data-id");

      // Try to find it from the cache
      if (previewCache.has(id)) {
        const cachedContent = previewCache.get(id);
        if (element.classList.contains("text-attachment")) {
          updateTextAttachment(id, cachedContent.content);
        } else if (element.classList.contains("pdf-attachment")) {
          updatePdfAttachment(id, cachedContent.objectUrl);
        } else if (element.classList.contains("image-attachment")) {
          console.log("Cached image: ", cachedContent);
          updateImageAttachment(id, cachedContent.base64, cachedContent.name);
        }
        // Fetch it newly
      } else {
        if (element.classList.contains("text-attachment")) {
          fetchPreview(url, id, "text");
        } else if (element.classList.contains("pdf-attachment")) {
          fetchPreview(url, id, "pdf");
        } else if (element.classList.contains("image-attachment")) {
          const name = element.getAttribute("name");
          console.log("Fetching image: ", url, name);
          fetchPreview(url, id, "image", name);
        }
      }
      element.dataset.initialized = true;
    }
  });
}

async function fetchPreview(url, id, type, opt = "") {
  try {
    const response = await fetch(url, { cache: "force-cache" });
    if (!response.ok) {
      throw new Error(`Failed to fetch preview for ${id}: ${response.statusText}`);
    }

    let content;
    if (type === "text") {
      content = await response.text();
      updateTextAttachment(id, content);
      previewCache.set(id, { type: "text", content });
    } else if (type === "pdf" || type === "image") {
      content = await response.blob();
      const objectUrl = URL.createObjectURL(content);
      if (type === "pdf") {
        updatePdfAttachment(id, objectUrl);
        previewCache.set(id, { type, objectUrl });
      } else if (type === "image") {
        console.log("image url and content: ", url, content);
        const base64 = await blobToBase64(content);
        updateImageAttachment(id, base64);
        previewCache.set(id, { type, base64, name: opt });
      }

    }
  } catch (error) {
    console.error("Error fetching preview:", error);
  }
}

function blobToBase64(blob) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => resolve(reader.result);
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}

function updateTextAttachment(id, content) {
  const element = window.content.querySelector(`.text-attachment[data-id='${id}']`);
  if (element) {
    element.innerHTML = `<pre>${content}</pre>`;
  }
}

function updatePdfAttachment(id, pdfUrl) {
  const element = window.content.querySelector(`.pdf-attachment[data-id='${id}']`);
  if (element) {
    element.innerHTML = `<iframe src="${pdfUrl}" type="application/pdf"></iframe>`;
  }
}

function updateImageAttachment(id, base64, name) {
  const element = window.content.querySelector(`.image-attachment[data-id='${id}']`);
  if (element) {
    element.innerHTML = `<img src="${base64}" alt="${name}"/>`;
  }
}



// Cache repositories
const previewCache = new Map();
