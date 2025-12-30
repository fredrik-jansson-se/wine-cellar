(() => {
  const wrap = document.getElementById("imgWrap");
  const img  = document.getElementById("targetImg");
  const rect = document.getElementById("rect");

  const outX = document.getElementById("roiX");
  const outY = document.getElementById("roiY");
  const outW = document.getElementById("roiW");
  const outH = document.getElementById("roiH");

  const submitBtn = document.getElementById("submitRoi");
  const clearBtn  = document.getElementById("clearRoi");
  const hint      = document.getElementById("roiHint");

  let dragging = false;
  let start = null;
  let hasROI = false;

  function clamp(v, min, max) { return Math.max(min, Math.min(max, v)); }

  function getLocalPoint(evt) {
    const r = img.getBoundingClientRect();
    const x = clamp(evt.clientX - r.left, 0, r.width);
    const y = clamp(evt.clientY - r.top,  0, r.height);
    return { x, y, bounds: r };
  }

  function toNatural(displayX, displayY, bounds) {
    const scaleX = img.naturalWidth  / bounds.width;
    const scaleY = img.naturalHeight / bounds.height;
    return {
      x: Math.round(displayX * scaleX),
      y: Math.round(displayY * scaleY),
    };
  }

  function drawRect(left, top, w, h) {
    rect.style.left = left + "px";
    rect.style.top  = top  + "px";
    rect.style.width  = w + "px";
    rect.style.height = h + "px";
    rect.style.display = (w > 2 && h > 2) ? "block" : "none";
  }

  function setButtons(enabled) {
    submitBtn.disabled = !enabled;
    clearBtn.disabled = !enabled;
    hint.textContent = enabled
      ? `Selected: x=${outX.value}, y=${outY.value}, w=${outW.value}, h=${outH.value} (pixels)`
      : "Drag on the image to select a region.";
  }

  function clearROI() {
    hasROI = false;
    outX.value = outY.value = outW.value = outH.value = "";
    rect.style.display = "none";
    setButtons(false);
  }

  clearBtn.addEventListener("click", clearROI);

  wrap.addEventListener("mousedown", (evt) => {
    if (evt.button !== 0) return;
    dragging = true;
    const p = getLocalPoint(evt);
    start = { x: p.x, y: p.y, bounds: p.bounds };
    // start drawing
    drawRect(start.x, start.y, 0, 0);
    evt.preventDefault();
  });

  window.addEventListener("mousemove", (evt) => {
    if (!dragging || !start) return;
    const p = getLocalPoint(evt);

    const left = Math.min(start.x, p.x);
    const top  = Math.min(start.y, p.y);
    const w    = Math.abs(p.x - start.x);
    const h    = Math.abs(p.y - start.y);

    drawRect(left, top, w, h);
  });

  window.addEventListener("mouseup", (evt) => {
    if (!dragging || !start) return;
    dragging = false;

    const end = getLocalPoint(evt);

    const left = Math.min(start.x, end.x);
    const top  = Math.min(start.y, end.y);
    const w    = Math.abs(end.x - start.x);
    const h    = Math.abs(end.y - start.y);

    if (w < 3 || h < 3) {
      // too small, keep previous ROI if any
      if (!hasROI) clearROI();
      start = null;
      return;
    }

    // Convert display rect to natural image pixels
    const p1 = toNatural(left, top, start.bounds);
    const p2 = toNatural(left + w, top + h, start.bounds);

    outX.value = p1.x;
    outY.value = p1.y;
    outW.value = Math.max(1, p2.x - p1.x);
    outH.value = Math.max(1, p2.y - p1.y);

    hasROI = true;
    setButtons(true);
    start = null;
  });

  // Optional: prevent submitting if no ROI (extra safety)
  document.getElementById("roiForm").addEventListener("submit", (e) => {
    if (!hasROI) e.preventDefault();
  });
})();
