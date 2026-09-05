import { Fragment as e, computed as t, createCommentVNode as n, createElementBlock as r, createElementVNode as i, defineComponent as a, onBeforeUnmount as o, onMounted as s, openBlock as c, ref as l, renderList as u, shallowRef as d, toDisplayString as f, vModelText as p, withDirectives as ee, withKeys as te } from "vue";
//#region ../.build/synth/web/bridge.js
async function m(e) {
	let { instance: t } = await WebAssembly.instantiate(e, {}), n = t.exports;
	for (let e of h) if (typeof n[e] != "function") throw Error(`the module does not export ${e}; the page and the module disagree`);
	let r = typeof n[g] == "function";
	r && n[g]();
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
var h = [
	"ess_input_reserve",
	"ess_dispatch",
	"ess_output_len"
], g = "ess_realize", _ = class extends Error {}, v = class e {
	#e;
	constructor(e) {
		this.#e = e;
	}
	static async open(t) {
		let n = await m(t);
		if (!n.realized) throw new _("this module exports no `ess_realize`, so every command would refuse as an unmet obligation — build `softphone-shell`, not the generated `softphone-web`");
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
		if (!n.ok) throw new _(`\`${e}\` was not accepted: ${JSON.stringify(n.error)}`);
		return n;
	}
};
function y(e, t, n) {
	let r = (e.outcome?.published ?? []).find((e) => e.event === t)?.payload[n];
	return typeof r == "string" ? r : void 0;
}
function b(e, t) {
	return e.views[t]?.rows ?? [];
}
//#endregion
//#region src/pcmu-first.mjs
function x(e, t) {
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
function S(e) {
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
function C(e) {
	let t = e.split(/(?<=\n)/), n = S(t), r = t[n].match(/\r?\n$/)?.[0] ?? "", i = t[n].slice(0, -r.length || void 0).split(" ").slice(3), a = x(t, n), o = (e) => a.find((t) => t.payload === e), s = (e, t, n, r) => e !== void 0 && e.name.toLowerCase() === t.toLowerCase() && e.clock === String(n) && e.channels === r, c = {
		"opus/48000/2": i.filter((e) => s(o(e), "opus", 48e3, "2")).length === 1,
		"PCMU at payload 0": i.includes("0"),
		"PCMA at payload 8": i.includes("8") && s(o("8"), "PCMA", 8e3, void 0),
		"CN at payload 13": i.includes("13") && s(o("13"), "CN", 8e3, void 0),
		"telephone-event/8000 with no channel count": i.filter((e) => s(o(e), "telephone-event", 8e3, void 0)).length === 1
	}, l = Object.entries(c).flatMap(([e, t]) => t ? [] : [e]);
	return l.length > 0 ? `the server's audio profile requires, and this offer has no ${l.join(", no ")}` : i.length < 5 ? `the offer names ${i.length} formats and the profile requires all five` : new Set(i).size === i.length ? null : "the offer lists a payload number twice, which the profile refuses";
}
function w(e) {
	let t = e.split(/(?<=\n)/), n = 0, r = t.filter((e) => {
		if (!e.startsWith("a=candidate:")) return !0;
		let t = e.slice(12).trim().split(" ")[2], r = t !== void 0 && t.toLowerCase() === "udp";
		return r && (n += 1), r;
	});
	if (n === 0) throw Error("this offer carries no UDP candidate, so there is no address the server could check");
	return r.join("");
}
function T(e) {
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
function E(e) {
	let t = e.split(/(?<=\n)/), n = S(t), r = t[n].match(/\r?\n$/)?.[0] ?? "", [i, a, o, ...s] = t[n].slice(0, -r.length || void 0).split(" ");
	if (!s.includes("0")) {
		let e = x(t, n).find((e) => e.name.toLowerCase() === "pcmu");
		throw Error(e ? `the offer names PCMU at payload ${e.payload}, and the server resolves PCMU only at payload 0, so putting it first would be answered and then refused` : "the offer's m=audio has no payload 0, so there is no PCMU for the server to select");
	}
	let c = x(t, n).find((e) => e.payload === "0");
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
async function D(e = {}) {
	let t = await navigator.mediaDevices.getUserMedia({ audio: !0 }), n;
	try {
		n = new RTCPeerConnection(e);
		for (let e of t.getAudioTracks()) n.addTrack(e, t);
		let { sdp: r } = await n.createOffer();
		return await n.setLocalDescription({
			type: "offer",
			sdp: E(r)
		}), await O(n), {
			connection: n,
			stream: t,
			sdp: w(n.localDescription.sdp)
		};
	} catch (e) {
		for (let e of t.getTracks()) e.stop();
		throw n?.close(), e;
	}
}
function O(e) {
	return e.iceGatheringState === "complete" ? Promise.resolve() : new Promise((t) => {
		e.addEventListener("icegatheringstatechange", function n() {
			e.iceGatheringState === "complete" && (e.removeEventListener("icegatheringstatechange", n), t());
		});
	});
}
//#endregion
//#region src/transport.ts
var k = class e {
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
}, A = {
	sample_format: "PcmS16Le",
	sample_rate_hz: 8e3,
	channels: 1,
	packet_time_ms: 20,
	frame_bytes: 320
}, j = class {
	#e;
	#t = null;
	#n;
	#r;
	#i = null;
	#a = null;
	constructor(e, t, n) {
		this.#e = e, this.#r = t, this.#n = n;
	}
	observe() {
		return this.#e.observe();
	}
	get busy() {
		return this.#a !== null;
	}
	configure(e) {
		if (this.#i) return;
		let t = this.#e.run("softphone.control.ConfigureEndpoint", {
			label: e,
			default_binding: "Bridge"
		});
		this.#i = y(t, "softphone.control.EndpointConfigured", "endpoint_id") ?? null, this.#n.observed(t);
	}
	async dial(e, t = {}) {
		if (!this.#i) throw Error("configure the phone before dialling");
		if (this.#a) throw Error("this phone is already holding a call");
		let { connection: n, stream: r, sdp: i } = await D(t), a = C(i) ?? T(i);
		if (a) throw this.#f(n, r), Error(`this browser's offer cannot be answered: ${a}`);
		let o = await this.#o().catch((e) => {
			throw this.#f(n, r), e;
		}), s = this.#e.run("softphone.media.OpenSession", {
			session_ref: `bridge-${Date.now().toString(36)}`,
			binding: "Bridge",
			profile: A,
			participant: {
				reference: e,
				trust: "Untrusted"
			}
		}), c = y(s, "softphone.media.SessionOpened", "session_id");
		this.#n.observed(s);
		let l = this.#e.run("softphone.bridge.ConnectBridge", {
			session_id: c,
			endpoint: this.#r,
			offer: i
		}), u = y(l, "softphone.bridge.BridgeConnecting", "bridge_id");
		this.#n.observed(l);
		let d = this.#e.run("softphone.control.Dial", {
			endpoint_id: this.#i,
			remote: e
		}), f = y(d, "softphone.control.CallDialled", "call_id");
		if (this.#n.observed(d), !c || !u || !f) throw this.#f(n, r), Error("the module refused part of the sequence; the log says which");
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
		this.#a = p, n.addEventListener("connectionstatechange", () => this.#c(p)), n.addEventListener("track", (e) => this.#u(e)), o.send({
			leg: "open-bridge",
			bridge_id: u,
			session_id: c,
			call_id: f,
			destination: e,
			offer: i
		});
	}
	hangUp() {
		let e = this.#a;
		e && (this.#n.observed(this.#e.run("softphone.control.HangUp", { call_id: e.call_id })), this.#t?.open && this.#t.send({
			leg: "hangup",
			call_id: e.call_id
		}), this.#d(e, "Local", "Completed"));
	}
	setMuted(e) {
		let t = this.#a;
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
		let t = this.#a;
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
		let t = this.#a;
		t && (this.#n.observed(this.#e.run("softphone.control.SendDigits", {
			call_id: t.call_id,
			digits: e
		})), this.#t?.send({
			leg: "digits",
			call_id: t.call_id,
			digits: e
		}));
	}
	async #o() {
		return this.#t?.open || (this.#t = await k.connect(this.#r, {
			command: (e) => this.#s(e),
			closed: (e) => this.#p(e)
		})), this.#t;
	}
	#s(e) {
		let t = this.#a;
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
		this.#n.observed(this.#e.run(e.command, e.input)), t && (e.command === "softphone.control.ConfirmAnswer" && (t.answered = !0, this.#l(t)), (e.command === "softphone.control.FailCall" || e.command === "softphone.bridge.FailBridge") && (this.#f(t.connection, t.stream), this.#a = null));
	}
	#c(e) {
		if (this.#a !== e) return;
		let t = e.connection.connectionState;
		if (t === "connected") {
			e.media = !0, this.#l(e);
			return;
		}
		(t === "failed" || t === "closed") && this.#n.said(`the media path is ${t}; the server will report the call`);
	}
	#l(e) {
		!e.activated && e.answered && e.media && (e.activated = !0, this.#n.observed(this.#e.run("softphone.control.AttachMedia", {
			call_id: e.call_id,
			session_id: e.session_id
		})), this.#n.observed(this.#e.run("softphone.control.MediaConnected", { call_id: e.call_id })));
	}
	#u(e) {
		let [t] = e.streams;
		if (!t) return;
		let n = document.getElementById("phone-far-end");
		n || (n = document.createElement("audio"), n.id = "phone-far-end", n.autoplay = !0, document.body.append(n)), n.srcObject = t;
	}
	#d(e, t, n) {
		this.#n.observed(this.#e.run("softphone.bridge.CloseBridge", {
			bridge_id: e.bridge_id,
			session_id: e.session_id,
			cause: t,
			reason: n
		})), this.#f(e.connection, e.stream), this.#a = null;
	}
	#f(e, t) {
		for (let e of t.getTracks()) e.stop();
		e.close();
	}
	#p(e) {
		this.#n.said(e);
		let t = this.#a;
		t && this.#d(t, "Transport", "TransportLost");
	}
}, ne = { class: "phone" }, re = {
	key: 0,
	class: "phone-failed"
}, M = { class: "phone-bezel" }, N = { class: "phone-screen" }, P = { class: "phone-state" }, F = { class: "phone-remote" }, I = { class: "phone-flags" }, L = { key: 0 }, R = { key: 1 }, z = {
	key: 1,
	class: "phone-idle"
}, B = { class: "phone-entry" }, V = { class: "phone-pad" }, H = ["onClick"], U = { class: "phone-to" }, W = { class: "phone-actions" }, G = ["disabled"], K = ["disabled"], q = ["disabled"], J = ["disabled"], Y = ["disabled"], ie = {
	key: 0,
	class: "phone-said"
}, X = { class: "phone-beside" }, ae = {
	key: 0,
	class: "phone-idle"
}, oe = { key: 1 }, se = { class: "phone-mono" }, ce = { class: "phone-log" }, Z = /*#__PURE__*/ ((e, t) => {
	let n = e.__vccOpts || e;
	for (let [e, r] of t) n[e] = r;
	return n;
})(/* @__PURE__ */ a({
	__name: "PhonePanel",
	props: {
		wasm: {},
		endpoint: { default: "ws://127.0.0.1:8780" },
		label: { default: "Devcenter" },
		configuration: { default: () => ({}) },
		destination: { default: "" }
	},
	setup(a, { expose: m }) {
		let h = a, g = d(null), _ = l(null), y = l(""), b = l(h.destination), x = l(""), S = l(""), C = () => {}, w = new Promise((e) => {
			C = e;
		}), T = [
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
		function E(e) {
			return _.value?.views[e]?.rows ?? [];
		}
		let D = t(() => E("softphone.control.CallById").find((e) => e.state !== "Ended")), O = t(() => D.value !== void 0), k = t(() => D.value?.muted === !0), A = t(() => D.value?.held === !0), Z = t(() => E("softphone.history.RecentCalls")), le = t(() => (_.value?.log ?? []).slice(-8).reverse());
		s(async () => {
			try {
				let e = typeof h.wasm == "string" ? await (await fetch(h.wasm)).arrayBuffer() : h.wasm, t = new j(await v.open(e), h.endpoint, {
					observed: (e) => _.value = e,
					said: (e) => y.value = e
				});
				t.configure(h.label), g.value = t, _.value = t.observe();
			} catch (e) {
				S.value = String(e);
			} finally {
				C();
			}
		}), o(() => g.value?.hangUp());
		async function Q(e) {
			y.value = "";
			try {
				await e();
			} catch (e) {
				y.value = String(e);
			}
		}
		let $ = () => Q(async () => {
			let e = b.value.trim() || x.value.trim();
			if (!e) throw Error("nothing to dial");
			await g.value?.dial(e, h.configuration);
		});
		m({
			dial: $,
			ready: w
		});
		let ue = (e) => {
			x.value += e, O.value && Q(() => g.value?.sendDigits(e));
		};
		return (t, a) => (c(), r("section", ne, [S.value ? (c(), r("p", re, f(S.value), 1)) : (c(), r(e, { key: 1 }, [i("div", M, [
			i("div", N, [D.value ? (c(), r(e, { key: 0 }, [
				i("p", P, f(D.value.state), 1),
				i("p", F, f(D.value.remote), 1),
				i("p", I, [k.value ? (c(), r("span", L, "muted")) : n("", !0), A.value ? (c(), r("span", R, "held")) : n("", !0)])
			], 64)) : (c(), r("p", z, "no call")), i("p", B, f(x.value || "—"), 1)]),
			i("div", V, [(c(), r(e, null, u(T, (e) => i("button", {
				key: e,
				type: "button",
				onClick: (t) => ue(e)
			}, f(e), 9, H)), 64))]),
			i("label", U, [a[5] ||= i("span", null, "call", -1), ee(i("input", {
				"onUpdate:modelValue": a[0] ||= (e) => b.value = e,
				type: "text",
				placeholder: "sip:1001@…",
				onKeyup: te($, ["enter"])
			}, null, 544), [[p, b.value]])]),
			i("div", W, [
				i("button", {
					type: "button",
					disabled: O.value || !g.value,
					onClick: $
				}, "dial", 8, G),
				i("button", {
					type: "button",
					disabled: !O.value,
					onClick: a[1] ||= (e) => Q(() => g.value?.hangUp())
				}, " hang up ", 8, K),
				i("button", {
					type: "button",
					disabled: !O.value,
					onClick: a[2] ||= (e) => Q(() => g.value?.setMuted(!k.value))
				}, f(k.value ? "unmute" : "mute"), 9, q),
				i("button", {
					type: "button",
					disabled: !O.value,
					onClick: a[3] ||= (e) => Q(() => g.value?.setHeld(!A.value))
				}, f(A.value ? "resume" : "hold"), 9, J),
				i("button", {
					type: "button",
					disabled: !x.value,
					onClick: a[4] ||= (e) => x.value = ""
				}, "clear", 8, Y)
			]),
			y.value ? (c(), r("p", ie, f(y.value), 1)) : n("", !0)
		]), i("div", X, [
			a[6] ||= i("h3", null, "calls", -1),
			Z.value.length ? (c(), r("table", oe, [i("tbody", null, [(c(!0), r(e, null, u(Z.value, (e) => (c(), r("tr", { key: String(e.record_id) }, [
				i("td", null, f(e.direction), 1),
				i("td", se, f(e.remote), 1),
				i("td", null, f(e.termination), 1)
			]))), 128))])])) : (c(), r("p", ae, "nothing recorded yet")),
			a[7] ||= i("h3", null, "log", -1),
			i("ol", ce, [(c(!0), r(e, null, u(le.value, (e) => (c(), r("li", {
				key: e.occurrence,
				class: "phone-mono"
			}, f(e.event.replace("softphone.", "")), 1))), 128))])
		])], 64))]));
	}
}), [["__scopeId", "data-v-6560c618"]]);
//#endregion
export { _ as DispatchFailed, v as Kernel, k as Link, j as Phone, Z as PhonePanel, D as createPcmuFirstOffer, E as pcmuFirst, y as published, b as rows, w as udpCandidatesOnly, C as unanswerable, T as uncheckable };

//# sourceMappingURL=index.js.map