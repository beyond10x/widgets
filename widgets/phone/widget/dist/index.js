import { Fragment as e, computed as t, createCommentVNode as n, createElementBlock as r, createElementVNode as i, createTextVNode as a, defineComponent as o, onBeforeUnmount as s, onMounted as c, openBlock as l, ref as u, renderList as d, shallowRef as f, toDisplayString as p, vModelText as m, withDirectives as h, withKeys as g } from "vue";
//#region ../.build/synth/web/bridge.js
async function _(e) {
	let { instance: t } = await WebAssembly.instantiate(e, {}), n = t.exports;
	for (let e of v) if (typeof n[e] != "function") throw Error(`the module does not export ${e}; the page and the module disagree`);
	let r = typeof n[y] == "function";
	r && n[y]();
	let i = new TextEncoder(), a = new TextDecoder();
	function o(e) {
		let t = i.encode(JSON.stringify(e)), r = n.ess_input_reserve(t.length);
		new Uint8Array(n.memory.buffer, r, t.length).set(t);
		let o = n.ess_dispatch(), s = n.ess_output_len(), c = a.decode(new Uint8Array(n.memory.buffer, o, s));
		return JSON.parse(c);
	}
	return {
		request: o,
		realized: r,
		exports: n
	};
}
var v = [
	"ess_input_reserve",
	"ess_dispatch",
	"ess_output_len"
], y = "ess_realize", b = class extends Error {}, x = class e {
	#e;
	constructor(e) {
		this.#e = e;
	}
	static async open(t) {
		let n = await _(t);
		if (!n.realized) throw new b("this module exports no `ess_realize`, so every command would refuse as an unmet obligation — build `softphone-shell`, not the generated `softphone-web`");
		return new e(n);
	}
	catalog() {
		return this.#e.request({ request: "catalog" }).catalog ?? {};
	}
	observe() {
		return this.#e.request({ request: "observe" });
	}
	run(e, t) {
		let n = this.#e.request({
			request: "command",
			command: e,
			input: t
		});
		if (!n.ok) throw new b(`\`${e}\` was not accepted: ${JSON.stringify(n.error)}`);
		return n;
	}
};
function S(e, t, n) {
	let r = (e.outcome?.published ?? []).find((e) => e.event === t)?.payload[n];
	return typeof r == "string" ? r : void 0;
}
function C(e, t) {
	return e.views[t]?.rows ?? [];
}
//#endregion
//#region src/pcmu-first.mjs
function w(e, t) {
	return e.slice(t + 1).flatMap((e) => {
		let t = e.match(/^a=rtpmap:(\d+) ([^/\r\n]+)\/(\d+)(\/(\d+))?/);
		return t ? [{
			payload: t[1],
			name: t[2],
			clock: t[3],
			channels: t[5]
		}] : [];
	});
}
function T(e) {
	let t = e.flatMap((e, t) => e.startsWith("m=") ? [t] : []);
	if (t.length === 0) throw Error("the offer has no m=audio section, and the page's offer is exactly one");
	if (t.length > 1) {
		let n = t.map((t) => e[t].split(" ")[0].slice(2));
		throw Error(`the offer has ${t.length} media sections (${n.join(", ")}); the page's offer is exactly one, m=audio`);
	}
	let [n] = t;
	if (!e[n].startsWith("m=audio ")) {
		let t = e[n].split(" ")[0].slice(2);
		throw Error(`the offer's one media section is ${t}; the page's offer is m=audio`);
	}
	return n;
}
function E(e) {
	let t = e.split(/(?<=\n)/), n = T(t), r = t[n].match(/\r?\n$/)?.[0] ?? "", i = t[n].slice(0, -r.length || void 0).split(" ").slice(3), a = w(t, n), o = (e) => a.find((t) => t.payload === e), s = (e, t, n, r) => e !== void 0 && e.name.toLowerCase() === t.toLowerCase() && e.clock === String(n) && e.channels === r, c = {
		"opus/48000/2": i.filter((e) => s(o(e), "opus", 48e3, "2")).length === 1,
		"PCMU at payload 0": i.includes("0"),
		"PCMA at payload 8": i.includes("8") && s(o("8"), "PCMA", 8e3, void 0),
		"CN at payload 13": i.includes("13") && s(o("13"), "CN", 8e3, void 0),
		"telephone-event/8000 with no channel count": i.filter((e) => s(o(e), "telephone-event", 8e3, void 0)).length === 1
	}, l = Object.entries(c).flatMap(([e, t]) => t ? [] : [e]);
	return l.length > 0 ? `the server's audio profile requires, and this offer has no ${l.join(", no ")}` : i.length < 5 ? `the offer names ${i.length} formats and the profile requires all five` : new Set(i).size === i.length ? null : "the offer lists a payload number twice, which the profile refuses";
}
function D(e) {
	let t = e.split(/(?<=\n)/), n = 0, r = t.filter((e) => {
		if (!e.startsWith("a=candidate:")) return !0;
		let t = e.slice(12).trim().split(" ")[2], r = t !== void 0 && t.toLowerCase() === "udp";
		return r && (n += 1), r;
	});
	if (n === 0) throw Error("this offer carries no UDP candidate, so there is no address the server could check");
	return r.join("");
}
function O(e) {
	let t = e.split(/(?<=\n)/).filter((e) => e.startsWith("a=candidate:")).map((e) => e.slice(12).trim().split(" "));
	if (t.length === 0) return "the offer carries no ICE candidate, and the profile requires at least one";
	let n = t.filter((e) => e[2]?.toLowerCase() !== "udp");
	if (n.length > 0) {
		let e = [...new Set(n.map((e) => e[2]))].join(", ");
		return `the offer carries ${n.length} candidate(s) over ${e}, and the profile refuses a description containing one`;
	}
	let r = t.map((e) => e[e.indexOf("typ") + 1]), i = [...new Set(r.filter((e) => e !== "host" && e !== "srflx"))];
	return i.length > 0 ? `the offer carries ${i.join(", ")} candidate(s), and the profile takes only host and srflx` : t.length > 32 ? `the offer carries ${t.length} candidates and the profile takes at most 32` : null;
}
function k(e) {
	let t = e.split(/(?<=\n)/), n = T(t), r = t[n].match(/\r?\n$/)?.[0] ?? "", [i, a, o, ...s] = t[n].slice(0, -r.length || void 0).split(" ");
	if (!s.includes("0")) {
		let e = w(t, n).find((e) => e.name.toLowerCase() === "pcmu");
		throw Error(e ? `the offer names PCMU at payload ${e.payload}, and the server resolves PCMU only at payload 0, so putting it first would be answered and then refused` : "the offer's m=audio has no payload 0, so there is no PCMU for the server to select");
	}
	let c = w(t, n).find((e) => e.payload === "0");
	if (c && (c.name.toLowerCase() !== "pcmu" || c.clock !== "8000" || c.channels)) throw Error(`the offer maps payload 0 to ${c.name}/${c.clock}${c.channels ? `/${c.channels}` : ""}, and the server resolves payload 0 as PCMU only when it is PCMU/8000 with no channel count`);
	if (s[0] === "0") return e;
	let l = [...s];
	return l.splice(l.indexOf("0"), 1), t[n] = [
		i,
		a,
		o,
		"0",
		...l
	].join(" ") + r, t.join("");
}
//#endregion
//#region src/offer.mjs
async function A(e = {}) {
	let t = await navigator.mediaDevices.getUserMedia({ audio: !0 }), n;
	try {
		n = new RTCPeerConnection(e);
		for (let e of t.getAudioTracks()) n.addTrack(e, t);
		let { sdp: r } = await n.createOffer();
		return await n.setLocalDescription({
			type: "offer",
			sdp: k(r)
		}), await j(n), {
			connection: n,
			stream: t,
			sdp: D(n.localDescription.sdp)
		};
	} catch (e) {
		for (let e of t.getTracks()) e.stop();
		throw n?.close(), e;
	}
}
function j(e) {
	return e.iceGatheringState === "complete" ? Promise.resolve() : new Promise((t) => {
		e.addEventListener("icegatheringstatechange", function n() {
			e.iceGatheringState === "complete" && (e.removeEventListener("icegatheringstatechange", n), t());
		});
	});
}
//#endregion
//#region src/transport.ts
var M = class e {
	#e;
	#t;
	constructor(e, t) {
		this.#e = e, this.#t = t, e.addEventListener("message", (e) => this.#n(e)), e.addEventListener("close", (e) => t.closed(e.reason || `the channel closed (${e.code})`)), e.addEventListener("error", () => t.closed("the channel failed"));
	}
	static connect(t, n) {
		return new Promise((r, i) => {
			let a = new WebSocket(t);
			a.addEventListener("open", () => r(new e(a, n)), { once: !0 }), a.addEventListener("error", () => i(/* @__PURE__ */ Error(`no control channel at ${t}`)), { once: !0 });
		});
	}
	send(e) {
		this.#e.send(JSON.stringify(e));
	}
	close() {
		this.#e.close();
	}
	get open() {
		return this.#e.readyState === WebSocket.OPEN;
	}
	#n(e) {
		if (typeof e.data != "string") return;
		let t;
		try {
			t = JSON.parse(e.data);
		} catch {
			this.#t.closed("a frame on the control channel was not JSON");
			return;
		}
		let n = t;
		if (n.request !== "command" || typeof n.command != "string") {
			this.#t.closed(`a frame this page has no message for: ${e.data.slice(0, 120)}`);
			return;
		}
		this.#t.command({
			request: "command",
			command: n.command,
			input: n.input ?? {}
		});
	}
}, N = {
	sample_format: "PcmS16Le",
	sample_rate_hz: 8e3,
	channels: 1,
	packet_time_ms: 20,
	frame_bytes: 320
}, P = class {
	#e;
	#t = null;
	#n;
	#r;
	#i = null;
	#a = null;
	#o = null;
	constructor(e, t, n) {
		this.#e = e, this.#r = t, this.#n = n;
	}
	observe() {
		return this.#e.observe();
	}
	get busy() {
		return this.#o !== null;
	}
	configure(e) {
		if (this.#i) return;
		let t = this.#e.run("softphone.control.ConfigureEndpoint", {
			label: e,
			default_binding: "Bridge"
		});
		this.#i = S(t, "softphone.control.EndpointConfigured", "endpoint_id") ?? null, this.#n.observed(t);
	}
	async announce(e, t) {
		if (this.#a) throw Error("this phone has already claimed a handle");
		let n = await this.#s(), r = this.#e.run("softphone.presence.AnnouncePresence", {
			handle: e,
			label: t
		}), i = S(r, "softphone.presence.PresenceAnnounced", "presence_id");
		if (this.#n.observed(r), !i) throw Error("the module refused the announcement; the log says why");
		this.#a = i, n.send({
			leg: "announce",
			presence_id: i,
			handle: e,
			label: t
		});
	}
	withdraw() {
		let e = this.#a;
		e && (this.#a = null, this.#n.observed(this.#e.run("softphone.presence.WithdrawPresence", { presence_id: e })), this.#t?.send({
			leg: "withdraw",
			presence_id: e
		}));
	}
	async dial(e, t = {}) {
		if (!this.#i) throw Error("configure the phone before dialling");
		if (this.#o) throw Error("this phone is already holding a call");
		let { connection: n, stream: r, sdp: i } = await A(t), a = E(i) ?? O(i);
		if (a) throw this.#p(n, r), Error(`this browser's offer cannot be answered: ${a}`);
		let o = await this.#s().catch((e) => {
			throw this.#p(n, r), e;
		}), s = this.#e.run("softphone.media.OpenSession", {
			session_ref: `bridge-${Date.now().toString(36)}`,
			binding: "Bridge",
			profile: N,
			participant: {
				reference: e,
				trust: "Untrusted"
			}
		}), c = S(s, "softphone.media.SessionOpened", "session_id");
		this.#n.observed(s);
		let l = this.#e.run("softphone.bridge.ConnectBridge", {
			session_id: c,
			endpoint: this.#r,
			offer: i
		}), u = S(l, "softphone.bridge.BridgeConnecting", "bridge_id");
		this.#n.observed(l);
		let d = this.#e.run("softphone.control.Dial", {
			endpoint_id: this.#i,
			remote: e
		}), f = S(d, "softphone.control.CallDialled", "call_id");
		if (this.#n.observed(d), !c || !u || !f) throw this.#p(n, r), Error("the module refused part of the sequence; the log says which");
		let p = {
			call_id: f,
			bridge_id: u,
			session_id: c,
			connection: n,
			stream: r,
			answered: !1,
			media: !1,
			activated: !1
		};
		this.#o = p, n.addEventListener("connectionstatechange", () => this.#l(p)), n.addEventListener("track", (e) => this.#d(e)), o.send({
			leg: "open-bridge",
			bridge_id: u,
			session_id: c,
			call_id: f,
			destination: e,
			offer: i
		});
	}
	hangUp() {
		let e = this.#o;
		e && (this.#n.observed(this.#e.run("softphone.control.HangUp", { call_id: e.call_id })), this.#t?.open && this.#t.send({
			leg: "hangup",
			call_id: e.call_id
		}), this.#f(e, "Local", "Completed"));
	}
	setMuted(e) {
		let t = this.#o;
		t && (this.#n.observed(this.#e.run("softphone.control.SetMuted", {
			call_id: t.call_id,
			muted: e
		})), this.#t?.send({
			leg: "mute",
			call_id: t.call_id,
			muted: e
		}));
	}
	setHeld(e) {
		let t = this.#o;
		t && (this.#n.observed(this.#e.run("softphone.control.SetHeld", {
			call_id: t.call_id,
			held: e
		})), this.#t?.send({
			leg: "hold",
			call_id: t.call_id,
			held: e
		}));
	}
	sendDigits(e) {
		let t = this.#o;
		t && (this.#n.observed(this.#e.run("softphone.control.SendDigits", {
			call_id: t.call_id,
			digits: e
		})), this.#t?.send({
			leg: "digits",
			call_id: t.call_id,
			digits: e
		}));
	}
	async #s() {
		return this.#t?.open || (this.#t = await M.connect(this.#r, {
			command: (e) => this.#c(e),
			closed: (e) => this.#m(e)
		})), this.#t;
	}
	#c(e) {
		let t = this.#o;
		if (e.command === "softphone.bridge.ConfirmBridge" && t) {
			let n = e.input.answer;
			if (typeof n != "string") {
				this.#n.said("the server confirmed a bridge with no answer in it");
				return;
			}
			t.connection.setRemoteDescription({
				type: "answer",
				sdp: n
			}).then(() => this.#n.observed(this.#e.run(e.command, e.input))).catch((e) => {
				this.#n.said(`this browser would not take the answer: ${String(e)}`), this.hangUp();
			});
			return;
		}
		this.#n.observed(this.#e.run(e.command, e.input)), t && (e.command === "softphone.control.ConfirmAnswer" && (t.answered = !0, this.#u(t)), (e.command === "softphone.control.FailCall" || e.command === "softphone.bridge.FailBridge") && (this.#p(t.connection, t.stream), this.#o = null));
	}
	#l(e) {
		if (this.#o !== e) return;
		let t = e.connection.connectionState;
		if (t === "connected") {
			e.media = !0, this.#u(e);
			return;
		}
		(t === "failed" || t === "closed") && this.#n.said(`the media path is ${t}; the server will report the call`);
	}
	#u(e) {
		!e.activated && e.answered && e.media && (e.activated = !0, this.#n.observed(this.#e.run("softphone.control.AttachMedia", {
			call_id: e.call_id,
			session_id: e.session_id
		})), this.#n.observed(this.#e.run("softphone.control.MediaConnected", { call_id: e.call_id })));
	}
	#d(e) {
		let [t] = e.streams;
		if (!t) return;
		let n = document.getElementById("phone-far-end");
		n || (n = document.createElement("audio"), n.id = "phone-far-end", n.autoplay = !0, document.body.append(n)), n.srcObject = t;
	}
	#f(e, t, n) {
		this.#n.observed(this.#e.run("softphone.bridge.CloseBridge", {
			bridge_id: e.bridge_id,
			session_id: e.session_id,
			cause: t,
			reason: n
		})), this.#p(e.connection, e.stream), this.#o = null;
	}
	#p(e, t) {
		for (let e of t.getTracks()) e.stop();
		e.close();
	}
	#m(e) {
		this.#n.said(e);
		let t = this.#o;
		t && this.#f(t, "Transport", "TransportLost");
	}
}, ee = { class: "phone" }, te = {
	key: 0,
	class: "phone-failed"
}, ne = { class: "phone-bezel" }, re = { class: "phone-screen" }, ie = { class: "phone-state" }, ae = { class: "phone-remote" }, oe = { class: "phone-flags" }, se = { key: 0 }, ce = { key: 1 }, F = {
	key: 1,
	class: "phone-idle"
}, I = { class: "phone-entry" }, L = { class: "phone-pad" }, R = ["onClick"], z = { class: "phone-to" }, B = { class: "phone-actions" }, V = ["disabled"], H = ["disabled"], U = ["disabled"], W = ["disabled"], G = ["disabled"], le = {
	key: 0,
	class: "phone-said"
}, ue = { class: "phone-beside" }, de = {
	key: 0,
	class: "phone-announce"
}, fe = ["disabled"], pe = {
	key: 1,
	class: "phone-mono"
}, me = {
	key: 2,
	class: "phone-idle"
}, he = {
	key: 3,
	class: "phone-roster"
}, ge = ["onClick"], _e = { class: "phone-mono" }, ve = {
	key: 4,
	class: "phone-idle"
}, ye = { key: 5 }, be = { class: "phone-mono" }, K = { class: "phone-log" }, q = /*#__PURE__*/ ((e, t) => {
	let n = e.__vccOpts || e;
	for (let [e, r] of t) n[e] = r;
	return n;
})(/* @__PURE__ */ o({
	__name: "PhonePanel",
	props: {
		wasm: {},
		endpoint: { default: "ws://127.0.0.1:8780" },
		label: { default: "Devcenter" },
		configuration: { default: () => ({}) },
		destination: { default: "" },
		handle: { default: "" }
	},
	setup(o, { expose: _ }) {
		let v = o, y = f(null), b = u(null), S = u(""), C = u(v.destination), w = u(""), T = u(""), E = u(v.handle), D = () => {}, O = new Promise((e) => {
			D = e;
		}), k = [
			"1",
			"2",
			"3",
			"4",
			"5",
			"6",
			"7",
			"8",
			"9",
			"*",
			"0",
			"#"
		];
		function A(e) {
			return b.value?.views[e]?.rows ?? [];
		}
		let j = t(() => A("softphone.control.CallById").find((e) => e.state !== "Ended")), M = t(() => j.value !== void 0), N = t(() => j.value?.muted === !0), q = t(() => j.value?.held === !0), J = t(() => A("softphone.history.RecentCalls")), Y = t(() => A("softphone.presence.MyPresence")[0]), X = t(() => A("softphone.presence.PresentPhones")), xe = t(() => (b.value?.log ?? []).slice(-8).reverse());
		c(async () => {
			try {
				let e = typeof v.wasm == "string" ? await (await fetch(v.wasm)).arrayBuffer() : v.wasm, t = new P(await x.open(e), v.endpoint, {
					observed: (e) => b.value = e,
					said: (e) => S.value = e
				});
				t.configure(v.label), y.value = t, b.value = t.observe();
			} catch (e) {
				T.value = String(e);
			} finally {
				D();
			}
		}), s(() => y.value?.hangUp());
		async function Z(e) {
			S.value = "";
			try {
				await e();
			} catch (e) {
				S.value = String(e);
			}
		}
		let Q = () => Z(async () => {
			let e = E.value.trim();
			if (!e) throw Error("nothing to announce");
			await y.value?.announce(e, v.label);
		}), $ = () => Z(async () => {
			let e = C.value.trim() || w.value.trim();
			if (!e) throw Error("nothing to dial");
			await y.value?.dial(e, v.configuration);
		});
		_({
			dial: $,
			announce: Q,
			ready: O
		});
		let Se = (e) => {
			w.value += e, M.value && Z(() => y.value?.sendDigits(e));
		};
		return (t, o) => (l(), r("section", ee, [T.value ? (l(), r("p", te, p(T.value), 1)) : (l(), r(e, { key: 1 }, [i("div", ne, [
			i("div", re, [j.value ? (l(), r(e, { key: 0 }, [
				i("p", ie, p(j.value.state), 1),
				i("p", ae, p(j.value.remote), 1),
				i("p", oe, [N.value ? (l(), r("span", se, "muted")) : n("", !0), q.value ? (l(), r("span", ce, "held")) : n("", !0)])
			], 64)) : (l(), r("p", F, "no call")), i("p", I, p(w.value || "—"), 1)]),
			i("div", L, [(l(), r(e, null, d(k, (e) => i("button", {
				key: e,
				type: "button",
				onClick: (t) => Se(e)
			}, p(e), 9, R)), 64))]),
			i("label", z, [o[7] ||= i("span", null, "call", -1), h(i("input", {
				"onUpdate:modelValue": o[0] ||= (e) => C.value = e,
				type: "text",
				placeholder: "sip:1001@…",
				onKeyup: g($, ["enter"])
			}, null, 544), [[m, C.value]])]),
			i("div", B, [
				i("button", {
					type: "button",
					disabled: M.value || !y.value,
					onClick: $
				}, "dial", 8, V),
				i("button", {
					type: "button",
					disabled: !M.value,
					onClick: o[1] ||= (e) => Z(() => y.value?.hangUp())
				}, " hang up ", 8, H),
				i("button", {
					type: "button",
					disabled: !M.value,
					onClick: o[2] ||= (e) => Z(() => y.value?.setMuted(!N.value))
				}, p(N.value ? "unmute" : "mute"), 9, U),
				i("button", {
					type: "button",
					disabled: !M.value,
					onClick: o[3] ||= (e) => Z(() => y.value?.setHeld(!q.value))
				}, p(q.value ? "resume" : "hold"), 9, W),
				i("button", {
					type: "button",
					disabled: !w.value,
					onClick: o[4] ||= (e) => w.value = ""
				}, "clear", 8, G)
			]),
			S.value ? (l(), r("p", le, p(S.value), 1)) : n("", !0)
		]), i("div", ue, [
			o[8] ||= i("h3", null, "here", -1),
			Y.value ? (l(), r("p", pe, [a(p(Y.value.handle) + " ", 1), i("button", {
				type: "button",
				onClick: o[6] ||= (e) => Z(() => y.value?.withdraw())
			}, "withdraw")])) : (l(), r("div", de, [h(i("input", {
				"onUpdate:modelValue": o[5] ||= (e) => E.value = e,
				type: "text",
				placeholder: "you@phone.dev.test",
				onKeyup: g(Q, ["enter"])
			}, null, 544), [[m, E.value]]), i("button", {
				type: "button",
				disabled: !y.value,
				onClick: Q
			}, "announce", 8, fe)])),
			o[9] ||= i("h3", null, "other phones", -1),
			X.value.length ? (l(), r("ul", he, [(l(!0), r(e, null, d(X.value, (e) => (l(), r("li", { key: String(e.handle) }, [i("button", {
				type: "button",
				onClick: (t) => C.value = String(e.handle)
			}, p(e.label), 9, ge), i("span", _e, p(e.handle), 1)]))), 128))])) : (l(), r("p", me, "nobody else is here")),
			o[10] ||= i("h3", null, "calls", -1),
			J.value.length ? (l(), r("table", ye, [i("tbody", null, [(l(!0), r(e, null, d(J.value, (e) => (l(), r("tr", { key: String(e.record_id) }, [
				i("td", null, p(e.direction), 1),
				i("td", be, p(e.remote), 1),
				i("td", null, p(e.termination), 1)
			]))), 128))])])) : (l(), r("p", ve, "nothing recorded yet")),
			o[11] ||= i("h3", null, "log", -1),
			i("ol", K, [(l(!0), r(e, null, d(xe.value, (e) => (l(), r("li", {
				key: e.occurrence,
				class: "phone-mono"
			}, p(e.event.replace("softphone.", "")), 1))), 128))])
		])], 64))]));
	}
}), [["__scopeId", "data-v-702ce978"]]);
//#endregion
export { b as DispatchFailed, x as Kernel, M as Link, P as Phone, q as PhonePanel, A as createPcmuFirstOffer, k as pcmuFirst, S as published, C as rows, D as udpCandidatesOnly, E as unanswerable, O as uncheckable };

//# sourceMappingURL=index.js.map