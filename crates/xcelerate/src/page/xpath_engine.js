function(xpath, all){
  /* Self-contained XPath subset evaluator that pierces open shadow roots and
     same-origin iframe documents.

     Invoked as fn.call(contextNode, xpath, all). The receiver is an Element or
     a Document; the search space is the composed subtree rooted at it. A
     leading slash or double-slash is treated as descendant-or-self from the
     receiver (so with a Document context, '//button' searches the whole page).

     Traversal follows the composed tree: open shadow roots (el.shadowRoot) and
     same-origin iframe documents (el.contentDocument) are entered; closed
     shadow roots and cross-origin frames are skipped. Order within a host is
     its light children first, then its shadow children.

     Returns an Array of matching nodes when all is true (composed-tree order,
     no duplicates), otherwise the first node in that order or null.
     Predicates are evaluated per context node, matching native position()
     semantics such as '//li[2]'. Unsupported syntax throws so the caller can
     fall back to native document.evaluate. */

  const isWs = (c) => { const k = c.charCodeAt(0); return k === 32 || k === 9 || k === 10 || k === 13; };
  const isDigit = (c) => { const k = c.charCodeAt(0); return k >= 48 && k <= 57; };
  const isNameStart = (c) => { const k = c.charCodeAt(0); return (k >= 65 && k <= 90) || (k >= 97 && k <= 122) || c === '_'; };
  const isNameChar = (c) => { const k = c.charCodeAt(0); return (k >= 65 && k <= 90) || (k >= 97 && k <= 122) || (k >= 48 && k <= 57) || c === '_' || c === '-' || c === '.'; };

  // --- composed-tree navigation -------------------------------------------
  const nodeKids = (node) => {
    const out = [];
    if (!node) return out;
    const cn = node.childNodes;
    if (cn) { for (let i = 0; i < cn.length; i++) out.push(cn[i]); }
    if (node.shadowRoot && node.shadowRoot.childNodes) {
      const s = node.shadowRoot.childNodes;
      for (let i = 0; i < s.length; i++) out.push(s[i]);
    }
    if (node.contentDocument) out.push(node.contentDocument);
    return out;
  };
  const nodeParent = (node) => {
    if (!node) return null;
    const p = node.parentNode;
    if (!p) return null;
    if (p.nodeType === 11) return p.host || null; // ShadowRoot -> host
    return p;
  };
  const matchesTest = (node, test) => {
    if (test === 'node') return true;
    if (!node || node.nodeType !== 1) return false;
    if (test === '*') return true;
    return node.localName === test;
  };

  // --- predicate tokenizer / parser ---------------------------------------
  const tokenize = (s) => {
    const toks = [];
    let i = 0;
    while (i < s.length) {
      const c = s[i];
      if (isWs(c)) { i++; continue; }
      if (c === '=') { if (s[i + 1] === '=') { toks.push(['op', '=']); i += 2; } else { toks.push(['op', '=']); i++; } continue; }
      if (c === '!' && s[i + 1] === '=') { toks.push(['op', '!=']); i += 2; continue; }
      if (c === '<') { if (s[i + 1] === '=') { toks.push(['op', '<=']); i += 2; } else { toks.push(['op', '<']); i++; } continue; }
      if (c === '>') { if (s[i + 1] === '=') { toks.push(['op', '>=']); i += 2; } else { toks.push(['op', '>']); i++; } continue; }
      if (c === '(') { toks.push(['(']); i++; continue; }
      if (c === ')') { toks.push([')']); i++; continue; }
      if (c === ',') { toks.push([',']); i++; continue; }
      if (c === '@') { toks.push(['@']); i++; continue; }
      const k = c.charCodeAt(0);
      if (k === 39 || k === 34) { // single or double quote
        let j = i + 1, v = '';
        while (j < s.length && s.charCodeAt(j) !== k) { v += s[j]; j++; }
        if (j >= s.length) throw new Error('XC_XPATH: unterminated string literal in predicate');
        toks.push(['str', v]); i = j + 1; continue;
      }
      if (isDigit(c)) { let j = i; while (j < s.length && isDigit(s[j])) j++; toks.push(['num', parseInt(s.slice(i, j), 10)]); i = j; continue; }
      if (isNameStart(c)) { let j = i; while (j < s.length && isNameChar(s[j])) j++; toks.push(['name', s.slice(i, j)]); i = j; continue; }
      throw new Error('XC_XPATH: unsupported character in predicate: ' + c);
    }
    return toks;
  };

  const expect = (toks, st, kind) => {
    const tk = toks[st.i];
    if (!tk || tk[0] !== kind) throw new Error('XC_XPATH: expected ' + kind + ' in predicate');
    st.i++;
    return tk;
  };

  const parseOperand = (toks, st) => {
    const tk = toks[st.i];
    if (!tk) throw new Error('XC_XPATH: unexpected end of predicate');
    if (tk[0] === '@') { st.i++; const n = expect(toks, st, 'name'); return { o: 'attr', name: n[1] }; }
    if (tk[0] === 'str') { st.i++; return { o: 'str', v: tk[1] }; }
    if (tk[0] === 'num') { st.i++; return { o: 'num', v: tk[1] }; }
    if (tk[0] === 'name') {
      const fn = tk[1];
      st.i++;
      if (fn === 'position') { expect(toks, st, '('); expect(toks, st, ')'); return { o: 'position' }; }
      if (fn === 'text') { expect(toks, st, '('); expect(toks, st, ')'); return { o: 'text' }; }
      if (fn === 'normalize-space') { expect(toks, st, '('); const a = parseOperand(toks, st); expect(toks, st, ')'); return { o: 'norm', a: a }; }
      throw new Error('XC_XPATH: unsupported function: ' + fn);
    }
    throw new Error('XC_XPATH: unsupported predicate operand');
  };

  const parsePrimary = (toks, st) => {
    const tk = toks[st.i];
    if (tk && tk[0] === 'name' && (tk[1] === 'contains' || tk[1] === 'starts-with')) {
      const fn = tk[1];
      st.i++;
      expect(toks, st, '(');
      const a = parseOperand(toks, st);
      expect(toks, st, ',');
      const b = parseOperand(toks, st);
      expect(toks, st, ')');
      return { k: 'fn', name: fn, a: a, b: b };
    }
    const left = parseOperand(toks, st);
    const op = toks[st.i];
    if (op && op[0] === 'op') {
      st.i++;
      const right = parseOperand(toks, st);
      return { k: 'cmp', l: left, op: op[1], r: right };
    }
    if (left.o === 'attr') return { k: 'exists', name: left.name };
    if (left.o === 'num') return { k: 'num', v: left.v };
    if (left.o === 'text') return { k: 'cmp', l: left, op: '!=', r: { o: 'str', v: '' } };
    throw new Error('XC_XPATH: unsupported predicate');
  };

  const parseUnary = (toks, st) => {
    const tk = toks[st.i];
    if (tk && tk[0] === 'name' && tk[1] === 'not') {
      st.i++;
      expect(toks, st, '(');
      const e = parseOr(toks, st);
      expect(toks, st, ')');
      return { k: 'not', e: e };
    }
    if (tk && tk[0] === '(') {
      st.i++;
      const e = parseOr(toks, st);
      expect(toks, st, ')');
      return e;
    }
    return parsePrimary(toks, st);
  };
  const parseAnd = (toks, st) => {
    let l = parseUnary(toks, st);
    while (toks[st.i] && toks[st.i][0] === 'name' && toks[st.i][1] === 'and') {
      st.i++;
      l = { k: 'and', l: l, r: parseUnary(toks, st) };
    }
    return l;
  };
  const parseOr = (toks, st) => {
    let l = parseAnd(toks, st);
    while (toks[st.i] && toks[st.i][0] === 'name' && toks[st.i][1] === 'or') {
      st.i++;
      l = { k: 'or', l: l, r: parseAnd(toks, st) };
    }
    return l;
  };
  const parsePred = (s) => {
    const toks = tokenize(s);
    const st = { i: 0 };
    const e = parseOr(toks, st);
    if (st.i !== toks.length) throw new Error('XC_XPATH: trailing tokens in predicate');
    return e;
  };

  // --- predicate evaluation ------------------------------------------------
  const normalize = (v) => {
    const s = String(v);
    let out = '', prevSpace = true;
    for (let i = 0; i < s.length; i++) {
      const k = s.charCodeAt(i);
      if (k === 32 || k === 9 || k === 10 || k === 13) { if (!prevSpace) { out += ' '; prevSpace = true; } }
      else { out += s[i]; prevSpace = false; }
    }
    if (out.length && out.charCodeAt(out.length - 1) === 32) out = out.slice(0, out.length - 1);
    return out;
  };
  const directText = (n) => {
    if (!n || !n.childNodes) return null;
    for (let i = 0; i < n.childNodes.length; i++) { const c = n.childNodes[i]; if (c.nodeType === 3) return c.nodeValue; }
    return null;
  };
  const operandVal = (o, n, pos) => {
    if (o.o === 'attr') { if (!n || !n.getAttribute) return null; return n.getAttribute(o.name); }
    if (o.o === 'str') return o.v;
    if (o.o === 'num') return o.v;
    if (o.o === 'text') return directText(n);
    if (o.o === 'norm') { const v = operandVal(o.a, n, pos); return v === null ? null : normalize(v); }
    if (o.o === 'position') return pos;
    return null;
  };
  const cmpVals = (a, b, op) => {
    if (a === null || b === null) return false;
    let x = a, y = b;
    if (typeof a === 'number' || typeof b === 'number') { x = Number(a); y = Number(b); }
    if (op === '=') return x === y;
    if (op === '!=') return x !== y;
    if (op === '<') return x < y;
    if (op === '>') return x > y;
    if (op === '<=') return x <= y;
    return x >= y;
  };
  const evalPred = (p, n, pos) => {
    if (p.k === 'num') return pos === p.v;
    if (p.k === 'and') return evalPred(p.l, n, pos) && evalPred(p.r, n, pos);
    if (p.k === 'or') return evalPred(p.l, n, pos) || evalPred(p.r, n, pos);
    if (p.k === 'not') return !evalPred(p.e, n, pos);
    if (p.k === 'exists') return !!(n && n.hasAttribute && n.hasAttribute(p.name));
    if (p.k === 'fn') {
      const a = operandVal(p.a, n, pos), b = operandVal(p.b, n, pos);
      if (a === null || b === null) return false;
      const x = String(a), y = String(b);
      if (p.name === 'contains') return x.indexOf(y) >= 0;
      return y.length <= x.length && x.slice(0, y.length) === y;
    }
    if (p.k === 'cmp') return cmpVals(operandVal(p.l, n, pos), operandVal(p.r, n, pos), p.op);
    return false;
  };
  const applyPreds = (nodes, preds) => {
    let cur = nodes;
    for (let pi = 0; pi < preds.length; pi++) {
      const p = preds[pi], out = [];
      for (let i = 0; i < cur.length; i++) { if (evalPred(p, cur[i], i + 1)) out.push(cur[i]); }
      cur = out;
    }
    return cur;
  };

  // --- path parsing --------------------------------------------------------
  const parseStep = (expr, i) => {
    while (i < expr.length && isWs(expr[i])) i++;
    if (expr[i] === '.' && expr[i + 1] === '.') return { step: { axis: 'parent', test: 'node', preds: [] }, next: i + 2 };
    if (expr[i] === '.') return { step: { axis: 'self', test: 'node', preds: [] }, next: i + 1 };
    let test;
    if (expr[i] === '*') { test = '*'; i++; }
    else {
      let j = i;
      while (j < expr.length && isNameChar(expr[j])) j++;
      if (j === i) throw new Error('XC_XPATH: empty or unsupported step');
      test = expr.slice(i, j);
      i = j;
    }
    const preds = [];
    while (true) {
      while (i < expr.length && isWs(expr[i])) i++;
      if (expr[i] !== '[') break;
      let depth = 1, j = i + 1, q = -1;
      while (j < expr.length) {
        const k = expr.charCodeAt(j);
        if (q >= 0) { if (k === q) q = -1; }
        else if (k === 39 || k === 34) { q = k; }
        else if (k === 91) { depth++; }
        else if (k === 93) { depth--; if (depth === 0) break; }
        j++;
      }
      if (depth !== 0) throw new Error('XC_XPATH: unbalanced predicate brackets');
      preds.push(parsePred(expr.slice(i + 1, j)));
      i = j + 1;
    }
    return { step: { axis: 'child', test: test, preds: preds }, next: i };
  };

  const parsePath = (expr) => {
    const s = expr;
    let i = 0;
    while (i < s.length && isWs(s[i])) i++;
    const steps = [];
    if (s[i] === '/') {
      steps.push({ axis: 'descendant-or-self', test: 'node', preds: [] });
      if (s[i + 1] === '/') i += 2; else i += 1;
    }
    while (i < s.length) {
      while (i < s.length && isWs(s[i])) i++;
      if (i >= s.length) break;
      const r = parseStep(s, i);
      steps.push(r.step);
      i = r.next;
      while (i < s.length && isWs(s[i])) i++;
      if (i >= s.length) break;
      if (s[i] === '/') {
        if (s[i + 1] === '/') { steps.push({ axis: 'descendant-or-self', test: 'node', preds: [] }); i += 2; }
        else i += 1;
      } else {
        throw new Error('XC_XPATH: unsupported syntax near: ' + s.slice(i, i + 12));
      }
    }
    if (!steps.length) throw new Error('XC_XPATH: empty expression');
    return steps;
  };

  // --- step evaluation -----------------------------------------------------
  const evalStep = (contexts, step) => {
    const out = [], seen = new Set();
    const push = (n) => { if (!seen.has(n)) { seen.add(n); out.push(n); } };
    if (step.axis === 'self') { for (let i = 0; i < contexts.length; i++) push(contexts[i]); return out; }
    if (step.axis === 'parent') { for (let i = 0; i < contexts.length; i++) { const p = nodeParent(contexts[i]); if (p) push(p); } return out; }
    if (step.axis === 'child') {
      for (let ci = 0; ci < contexts.length; ci++) {
        const kids = nodeKids(contexts[ci]);
        let cand = [];
        for (let k = 0; k < kids.length; k++) { if (matchesTest(kids[k], step.test)) cand.push(kids[k]); }
        if (step.preds.length) cand = applyPreds(cand, step.preds);
        for (let k = 0; k < cand.length; k++) push(cand[k]);
      }
      return out;
    }
    // descendant-or-self (implicit for the double-slash abbreviation, no preds)
    const visit = (node) => {
      if (seen.has(node)) return;
      seen.add(node);
      if (matchesTest(node, step.test)) out.push(node);
      const kids = nodeKids(node);
      for (let k = 0; k < kids.length; k++) visit(kids[k]);
    };
    for (let i = 0; i < contexts.length; i++) visit(contexts[i]);
    return out;
  };

  const steps = parsePath(xpath);
  // Composed-order index (pre-order over light children, then shadow children,
  // then iframe documents) so the final node-set can be returned in document
  // order regardless of the order contexts happened to contribute matches.
  const composedRoot = (node) => {
    let r = node;
    while (true) {
      const rr = (r && r.getRootNode) ? r.getRootNode() : r;
      if (rr && rr.nodeType === 11 && rr.host) { r = rr.host; continue; }
      return rr;
    }
  };
  const orderIndex = new Map();
  let orderSeq = 0;
  const indexTree = (n) => {
    if (!n || orderIndex.has(n)) return;
    orderIndex.set(n, orderSeq++);
    const kids = nodeKids(n);
    for (let k = 0; k < kids.length; k++) indexTree(kids[k]);
  };
  indexTree(composedRoot(this));
  const byOrder = (a, b) => {
    const ia = orderIndex.has(a) ? orderIndex.get(a) : Infinity;
    const ib = orderIndex.has(b) ? orderIndex.get(b) : Infinity;
    return ia - ib;
  };
  let nodes = [this];
  for (let si = 0; si < steps.length; si++) {
    nodes = evalStep(nodes, steps[si]);
    if (!nodes.length) break;
  }
  if (nodes.length > 1) nodes = nodes.slice().sort(byOrder);
  return all ? nodes : (nodes[0] || null);
}
