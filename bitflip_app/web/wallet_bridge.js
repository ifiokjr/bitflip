"use strict";
(() => {
	var B = "solana:mainnet", z = "solana:devnet", U = "solana:testnet", j = "solana:localnet", b = [B, z, U, j];
	var f = "solana:signAndSendTransaction";
	var g = "solana:signMessage";
	var R = function(e, n, t, r) {
			if (t === "a" && !r) throw new TypeError("Private accessor was defined without a getter");
			if (typeof n == "function" ? e !== n || !r : !n.has(e)) {
				throw new TypeError("Cannot read private member from an object whose class did not declare it");
			}
			return t === "m" ? r : t === "a" ? r.call(e) : r ? r.value : n.get(e);
		},
		H = function(e, n, t, r, a) {
			if (r === "m") throw new TypeError("Private method is not writable");
			if (r === "a" && !a) throw new TypeError("Private accessor was defined without a setter");
			if (typeof n == "function" ? e !== n || !a : !n.has(e)) {
				throw new TypeError("Cannot write private member to an object whose class did not declare it");
			}
			return r === "a" ? a.call(e, t) : a ? a.value = t : n.set(e, t), t;
		},
		h,
		l,
		p = new Set();
	function q(e) {
		c = void 0, p.add(e);
	}
	function D(e) {
		c = void 0, p.delete(e);
	}
	var i = {};
	function W() {
		if (l || (l = Object.freeze({ register: E, get: V, on: Z }), typeof window > "u")) return l;
		let e = Object.freeze({ register: E });
		try {
			window.addEventListener("wallet-standard:register-wallet", ({ detail: n }) => n(e));
		} catch (n) {
			console.error(
				`wallet-standard:register-wallet event listener could not be added
`,
				n,
			);
		}
		try {
			window.dispatchEvent(new S(e));
		} catch (n) {
			console.error(
				`wallet-standard:app-ready event could not be dispatched
`,
				n,
			);
		}
		return l;
	}
	function E(...e) {
		return e = e.filter(n => !p.has(n)),
			e.length
				? (e.forEach(n => q(n)), i.register?.forEach(n => v(() => n(...e))), function() {
					e.forEach(t => D(t)), i.unregister?.forEach(t => v(() => t(...e)));
				})
				: () => {};
	}
	var c;
	function V() {
		return c || (c = [...p]), c;
	}
	function Z(e, n) {
		return i[e]?.push(n) || (i[e] = [n]), function() {
			i[e] = i[e]?.filter(r => n !== r);
		};
	}
	function v(e) {
		try {
			e();
		} catch (n) {
			console.error(n);
		}
	}
	var S = class extends Event {
		get detail() {
			return R(this, h, "f");
		}
		get type() {
			return "wallet-standard:app-ready";
		}
		constructor(n) {
			super("wallet-standard:app-ready", { bubbles: !1, cancelable: !1, composed: !1 }),
				h.set(this, void 0),
				H(this, h, n, "f");
		}
		preventDefault() {
			throw new Error("preventDefault cannot be called");
		}
		stopImmediatePropagation() {
			throw new Error("stopImmediatePropagation cannot be called");
		}
		stopPropagation() {
			throw new Error("stopPropagation cannot be called");
		}
	};
	h = new WeakMap();
	var m = "standard:connect";
	var C = "standard:events";
	var G = 2048,
		K = 4096,
		$ = 64,
		P = 64,
		_ = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz",
		k = W(),
		T = new WeakMap(),
		x = 1,
		u = null,
		d = null;
	function N(e) {
		if (!b.some(n => n === e)) throw new Error(`Unsupported Solana chain: ${e}`);
		return e;
	}
	function w(e) {
		return typeof e == "object" && e !== null;
	}
	function F(e, n) {
		if (!e.chains.includes(n)) return !1;
		let t = e.features[m], r = e.features[f], a = e.features[g];
		return w(t) && t.version === "1.0.0" && typeof t.connect == "function" && w(r) && r.version === "1.0.0"
			&& typeof r.signAndSendTransaction == "function" && Array.isArray(r.supportedTransactionVersions)
			&& r.supportedTransactionVersions.includes(0) && w(a) && (a.version === "1.0.0" || a.version === "1.1.0")
			&& typeof a.signMessage == "function";
	}
	function J(e) {
		if (e.length < 32 || e.length > 44) return null;
		let n = [0];
		for (let t of e) {
			let r = _.indexOf(t);
			if (r < 0) return null;
			let a = r;
			for (let o = 0; o < n.length; o += 1) {
				let s = n[o];
				if (s === void 0) return null;
				a += s * 58, n[o] = a & 255, a >>= 8;
			}
			for (; a > 0;) n.push(a & 255), a >>= 8;
		}
		for (let t = 0; t < e.length - 1 && e[t] === "1"; t += 1) n.push(0);
		return n.length !== 32 ? null : Uint8Array.from(n.reverse());
	}
	function A(e, n) {
		let t = J(e.address);
		return e.publicKey.byteLength === 32 && t !== null && I(t, Uint8Array.from(e.publicKey)) && e.chains.includes(n)
			&& e.features.includes(f) && e.features.includes(g);
	}
	function M(e) {
		return k.get().filter(n => F(n, e)).slice(0, $);
	}
	function O(e) {
		let n = T.get(e);
		if (n !== void 0) return n;
		let t = `wallet-${x}`;
		return x += 1, T.set(e, t), t;
	}
	function Q(e) {
		let n = N(e), t = M(n).map(r => ({ id: O(r), name: X(r.name) }));
		return JSON.stringify(t);
	}
	function X(e) {
		let n = e.replace(/[\u0000-\u001f\u007f]/g, "").trim().slice(0, 80);
		return n.length === 0 ? "Solana wallet" : n;
	}
	function L() {
		let e = u;
		if (e === null || !F(e.wallet, e.chain) || !A(e.account, e.chain)) {
			throw new Error("Connect a compatible wallet before signing.");
		}
		return e;
	}
	function Y(e) {
		d?.(), d = null;
		let n = e.wallet.features[C];
		if (!w(n) || typeof n.on != "function") return;
		d = n.on("change", r => {
			if (r.accounts === void 0 || u !== e) return;
			let a = r.accounts.find(o => o.address === e.account.address && A(o, e.chain));
			if (a === void 0) {
				u = null, d?.(), d = null;
				return;
			}
			e.account = a;
		});
	}
	async function ee(e, n) {
		let t = N(n), r = M(t).find(y => O(y) === e);
		if (r === void 0) throw new Error("The selected wallet is no longer available.");
		let o = (await r.features[m].connect()).accounts.find(y => A(y, t));
		if (o === void 0) throw new Error("The wallet did not authorize an account with the required Solana features.");
		let s = { wallet: r, account: o, chain: t };
		return u = s, Y(s), o.address;
	}
	function ne() {
		return u?.account.address ?? null;
	}
	function te(e) {
		if (
			e.length === 0 || e.length % 4 !== 0
			|| !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(e)
		) throw new Error("The transaction is not valid base64.");
		let n = atob(e);
		if (n.length === 0 || n.length > G) throw new Error("The serialized transaction has an invalid size.");
		return Uint8Array.from(n, t => t.charCodeAt(0));
	}
	function re(e) {
		let n = "";
		for (let t of e) n += String.fromCharCode(t);
		return btoa(n);
	}
	function ae(e) {
		let n = [0];
		for (let t of e) {
			let r = t;
			for (let a = 0; a < n.length; a += 1) {
				let o = n[a];
				if (o === void 0) throw new Error("Invalid base58 encoder state.");
				r += o << 8, n[a] = r % 58, r = Math.floor(r / 58);
			}
			for (; r > 0;) n.push(r % 58), r = Math.floor(r / 58);
		}
		for (let t = 0; t < e.length - 1 && e[t] === 0; t += 1) n.push(0);
		return n.reverse().map(t => _[t]).join("");
	}
	function I(e, n) {
		if (e.byteLength !== n.byteLength) return !1;
		let t = 0;
		for (let r = 0; r < e.byteLength; r += 1) t |= (e[r] ?? 0) ^ (n[r] ?? 0);
		return t === 0;
	}
	async function oe(e) {
		let n = L(),
			t = te(e),
			r = await n.wallet.features[f].signAndSendTransaction({
				account: n.account,
				chain: n.chain,
				transaction: t,
				options: { preflightCommitment: "confirmed" },
			});
		if (r.length !== 1 || r[0]?.signature.byteLength !== P) {
			throw new Error("The wallet returned an invalid transaction signature.");
		}
		return ae(r[0].signature);
	}
	async function ie(e) {
		let n = L(), t = new TextEncoder().encode(e);
		if (t.length === 0 || t.length > K) throw new Error("The message has an invalid size.");
		let r = await n.wallet.features[g].signMessage({ account: n.account, message: t }), a = r[0];
		if (
			r.length !== 1 || a === void 0 || a.signature.byteLength !== P
			|| a.signatureType !== void 0 && a.signatureType !== "ed25519" || !I(a.signedMessage, t)
		) throw new Error("The wallet returned an invalid message signature.");
		return re(a.signature);
	}
	globalThis.bitflipWallet = Object.freeze({
		isSupported: !0,
		address: ne,
		listWallets: Q,
		connect: ee,
		signAndSend: oe,
		signMessage: ie,
	});
})();
