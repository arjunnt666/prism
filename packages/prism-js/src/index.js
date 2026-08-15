function domains(snapshot) {
  return (snapshot.results || []).map((r) => r.domain);
}
function positions(snapshot) {
  const out = {};
  for (const r of snapshot.results || []) {
    const d = (r.domain || "").toLowerCase();
    if (d && out[d] === undefined) out[d] = r.position;
  }
  return out;
}
module.exports = { domains, positions, version: "0.1.0" };
