using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using BepInEx;
using Data.ScriptableObject;
using Game.Info;
using Manager;
using UnityEngine;
using UnityEngine.SceneManagement;

namespace SolarExpanseWikiDumper
{
    [BepInPlugin(PluginGuid, PluginName, PluginVersion)]
    public class Plugin : BaseUnityPlugin
    {
        internal const string PluginGuid = "com.aaronstockmeister.solar-expanse-wiki-dumper";
        private const string PluginName = "Solar Expanse Wiki Dumper";
        private const string PluginVersion = "0.1.0";

        internal const string OutputFileName = "sirenix-dump.json";
        internal const string MarkerFileName = "sirenix-dump.flag";

        internal static BepInEx.Logging.ManualLogSource Log;
        private static bool dumped;

        // Plugin.Update / Plugin.Start aren't ticked by Unity in this BepInEx setup
        // (verified: only Awake + OnEnable fire on BaseUnityPlugin and on a fresh
        // MonoBehaviour we spawn on a DontDestroyOnLoad GameObject). Instead, we
        // subscribe to the static SceneManager.sceneLoaded event in Awake — that
        // event is driven by Unity's scene-load pipeline, independent of our
        // GameObject's lifecycle, and fires every time a scene finishes loading.
        // When the gameplay scene loads, the ObjectInfo MonoBehaviours and the
        // AllScriptableObjectManager singleton are all present, so we can run the
        // dump directly from the handler.
        private void Awake()
        {
            Log = Logger;
            var marker = Path.Combine(Application.streamingAssetsPath, MarkerFileName);
            if (File.Exists(marker))
            {
                Log.LogInfo($"Marker present at {marker}; dumper will not run. Delete it to re-dump.");
                return;
            }
            SceneManager.sceneLoaded += OnSceneLoaded;
            Log.LogInfo("Subscribed to SceneManager.sceneLoaded. Waiting for the gameplay scene to load.");
        }

        private void OnSceneLoaded(Scene scene, LoadSceneMode mode)
        {
            Log.LogInfo($"sceneLoaded: name='{scene.name}' mode={mode} dumped={dumped}");
            if (dumped) return;

            // Gameplay-ready predicate: ObjectInfoManager exists AND its allObjectInfos
            // list is populated (SolarLoader.CreateSolarSystem calls SetAllObjectInfos
            // during gameplay-scene load). If we're not ready yet, just return —
            // sceneLoaded will fire again for the next scene.
            var oim = MonoBehaviourSingleton<ObjectInfoManager>.Instance;
            if (oim == null) { Log.LogInfo("  ObjectInfoManager.Instance is null; will wait for a later scene."); return; }
            var allField = typeof(ObjectInfoManager).GetField(
                "allObjectInfos", BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public);
            var all = allField?.GetValue(oim) as System.Collections.ICollection;
            if (all == null || all.Count == 0)
            {
                Log.LogInfo($"  allObjectInfos.count={(all == null ? -1 : all.Count)}; will wait for a later scene.");
                return;
            }
            var asoMgr = SerializedMonoBehaviourSingleton<AllScriptableObjectManager>.Instance
                         ?? UnityEngine.Object.FindObjectOfType<AllScriptableObjectManager>();
            if (asoMgr == null) { Log.LogInfo("  AllScriptableObjectManager not available; will wait."); return; }

            dumped = true;
            var dir = Application.streamingAssetsPath;
            try
            {
                Log.LogInfo($"  ready: ObjectInfoManager has {all.Count} bodies. Running dump.");
                var json = Dumper.Dump(asoMgr);
                File.WriteAllText(Path.Combine(dir, OutputFileName), json);
                File.WriteAllText(Path.Combine(dir, MarkerFileName), DateTime.UtcNow.ToString("O"));
                Log.LogInfo($"Wrote {json.Length:N0} characters to {OutputFileName}");
            }
            catch (Exception ex)
            {
                Log.LogError($"Dump failed: {ex}");
            }
        }
    }

    internal static class Dumper
    {
        // Class names whose instances we serialize.  These all live under
        // AllScriptableObjectManager's `allMyIDScriptableObjects` list at runtime,
        // plus a few config aggregators that are direct fields on the manager.
        private static readonly HashSet<string> DumpTypes = new HashSet<string>
        {
            "SpacecraftType",
            "LaunchVehicleType",
            "ContractDefinition",
            "ResourceDefinition",
            "CompanyDefinition",
            "EngineType",
            "ResearchDefinition",
            "ObjectType",
            "ObjectSubType",
            "ResearchType",
            "ResearchSubType",
            "PlanetarySystemDescriptor",
            "StartGameData",
            "StartGameEpoch",
            "FacilityDefinition",
            "Facility",
            // Facilities (FacilityBaseDescriptor subclasses):
            "GroundFacilityDescriptor",
            "SpaceModuleDescriptor",
            // Engine / tank / cargo / crew modules attached to spacecraft hulls:
            "SpaceComponent",
        };

        public static string Dump(AllScriptableObjectManager manager)
        {
            // 1. The manager exposes a List<MyIDScriptableObject> via a private field — grab it.
            var listField = typeof(AllScriptableObjectManager).GetField(
                "allMyIDScriptableObjects",
                BindingFlags.Instance | BindingFlags.NonPublic);
            var all = listField?.GetValue(manager) as System.Collections.IEnumerable;

            var byType = new SortedDictionary<string, List<UnityEngine.Object>>(StringComparer.Ordinal);
            if (all != null)
            {
                foreach (var obj in all)
                {
                    if (!(obj is UnityEngine.Object uo) || uo == null) continue;
                    var typeName = uo.GetType().Name;
                    if (!DumpTypes.Contains(typeName)) continue;
                    if (!byType.TryGetValue(typeName, out var list))
                    {
                        list = new List<UnityEngine.Object>();
                        byType[typeName] = list;
                    }
                    list.Add(uo);
                }
            }

            // 2. Sweep through Resources.FindObjectsOfTypeAll as a backstop — picks up
            //    types that aren't under MyIDScriptableObject (FacilityDefinition,
            //    EngineType, etc., which derive from MyIDScriptableObjectProductionItem).
            foreach (var so in Resources.FindObjectsOfTypeAll<ScriptableObject>())
            {
                if (so == null) continue;
                var typeName = so.GetType().Name;
                if (!DumpTypes.Contains(typeName)) continue;
                if (!byType.TryGetValue(typeName, out var list))
                {
                    list = new List<UnityEngine.Object>();
                    byType[typeName] = list;
                }
                if (!list.Contains(so)) list.Add(so);
            }

            // 3. Walk ObjectInfo MonoBehaviours.  ResourceMiningLicenseFeePerT is
            //    [OdinSerialize] and only populated at runtime, so it's invisible to
            //    AssetRipper / static dumps.  Values are per-body; identical across
            //    scenarios (scenarios change player state, not planet properties).
            var objectInfos = Resources.FindObjectsOfTypeAll<ObjectInfo>()
                .Where(oi => oi != null)
                .ToList();

            var writer = new JsonWriter();
            writer.StartObject();
            foreach (var kv in byType)
            {
                writer.Key(kv.Key);
                writer.StartArray();
                var visited = new HashSet<object>(new ReferenceEqualityComparer());
                foreach (var so in kv.Value)
                {
                    // Top-level: expand the SO's fields.  Nested references to other
                    // UnityEngine.Objects still emit as $ref via SerializeReflected.
                    ExpandComposite(writer, so, visited, depth: 0);
                }
                writer.EndArray();
            }

            // Emit the per-body license-fee table.  Shape:
            //   "ObjectInfo": [
            //     { "name": "Earth", "resourceMiningLicenseFeePerT": { "alloy": 30, ... } },
            //     { "name": "Mars",  "resourceMiningLicenseFeePerT": {} },
            //     ...
            //   ]
            // Resource keys come from the dictionary key's MyIDScriptableObject.ID
            // (falling back to the asset name) so they line up with the same id
            // space parse_sirenix.rs uses for every other resource reference.
            if (objectInfos.Count > 0)
            {
                writer.Key("ObjectInfo");
                writer.StartArray();
                foreach (var oi in objectInfos)
                {
                    writer.StartObject();
                    writer.Key("name");
                    writer.String(oi.gameObject ? oi.gameObject.name : oi.name);
                    writer.Key("resourceMiningLicenseFeePerT");
                    writer.StartObject();
                    foreach (var kv in oi.ResourceMiningLicenseFeePerT)
                    {
                        if (kv.Key == null) continue;
                        var resId = (kv.Key as MyIDScriptableObject)?.ID ?? kv.Key.name;
                        if (string.IsNullOrEmpty(resId)) continue;
                        writer.Key(resId);
                        writer.Raw(kv.Value.ToString(System.Globalization.CultureInfo.InvariantCulture));
                    }
                    writer.EndObject();
                    writer.EndObject();
                }
                writer.EndArray();
            }

            writer.EndObject();
            Plugin.Log.LogInfo($"Collected {byType.Sum(p => p.Value.Count):N0} objects across {byType.Count} type(s).");
            Plugin.Log.LogInfo($"Walked {objectInfos.Count} ObjectInfo MonoBehaviours for license fees.");
            return writer.ToString();
        }

        private const int MaxDepth = 6;

        private static void SerializeReflected(JsonWriter w, object obj, HashSet<object> visited, int depth)
        {
            if (obj == null) { w.Null(); return; }
            if (depth > MaxDepth) { w.String("[max-depth]"); return; }

            var t = obj.GetType();

            if (obj is string s) { w.String(s); return; }
            if (t.IsPrimitive)
            {
                if (obj is bool b) { w.Bool(b); return; }
                if (obj is char c) { w.String(c.ToString()); return; }
                w.Raw(Convert.ToString(obj, System.Globalization.CultureInfo.InvariantCulture));
                return;
            }
            if (t.IsEnum) { w.String(obj.ToString()); return; }
            if (obj is decimal d) { w.Raw(d.ToString(System.Globalization.CultureInfo.InvariantCulture)); return; }

            if (obj is UnityEngine.Object uObj)
            {
                w.StartObject();
                w.Key("$ref"); w.Bool(true);
                w.Key("type"); w.String(t.Name);
                w.Key("name"); w.String(uObj == null ? null : uObj.name ?? "");
                w.EndObject();
                return;
            }

            if (obj is System.Collections.IDictionary dict)
            {
                w.StartObject();
                foreach (var key in dict.Keys)
                {
                    w.Key(Convert.ToString(key, System.Globalization.CultureInfo.InvariantCulture) ?? "?");
                    SerializeReflected(w, dict[key], visited, depth + 1);
                }
                w.EndObject();
                return;
            }
            if (obj is System.Collections.IEnumerable enumerable)
            {
                w.StartArray();
                foreach (var item in enumerable)
                {
                    SerializeReflected(w, item, visited, depth + 1);
                }
                w.EndArray();
                return;
            }

            ExpandComposite(w, obj, visited, depth);
        }

        /// <summary>
        /// Walks public + [SerializeField] fields and writes them as a JSON object,
        /// without the UnityEngine.Object → $ref short-circuit.  Use for top-level
        /// entries; nested fields still go through SerializeReflected and emit refs.
        /// </summary>
        private static void ExpandComposite(JsonWriter w, object obj, HashSet<object> visited, int depth)
        {
            if (obj == null) { w.Null(); return; }
            if (!visited.Add(obj)) { w.String("[cycle]"); return; }
            try
            {
                w.StartObject();
                if (obj is UnityEngine.Object uObj)
                {
                    w.Key("$name"); w.String(uObj.name ?? "");
                    w.Key("$type"); w.String(obj.GetType().Name);
                }
                foreach (var field in CollectSerializedFields(obj.GetType()))
                {
                    object value;
                    try { value = field.GetValue(obj); }
                    catch { continue; }
                    w.Key(field.Name);
                    SerializeReflected(w, value, visited, depth + 1);
                }
                w.EndObject();
            }
            finally { visited.Remove(obj); }
        }

        private static IEnumerable<FieldInfo> CollectSerializedFields(Type t)
        {
            const BindingFlags F = BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.DeclaredOnly;
            for (var cur = t; cur != null && cur != typeof(object) && cur != typeof(UnityEngine.Object); cur = cur.BaseType)
            {
                foreach (var f in cur.GetFields(F))
                {
                    if (f.IsStatic || f.IsNotSerialized) continue;
                    var isPublic = f.IsPublic;
                    var hasSerializeField = f.GetCustomAttributes(typeof(SerializeField), true).Length > 0;
                    if (!isPublic && !hasSerializeField) continue;
                    yield return f;
                }
            }
        }
    }

    internal class ReferenceEqualityComparer : IEqualityComparer<object>
    {
        public new bool Equals(object x, object y) => ReferenceEquals(x, y);
        public int GetHashCode(object obj) => System.Runtime.CompilerServices.RuntimeHelpers.GetHashCode(obj);
    }

    internal class JsonWriter
    {
        private readonly StringBuilder sb = new StringBuilder();
        private readonly Stack<bool> firstStack = new Stack<bool>();
        private int indent;

        public void StartObject() { Comma(); Indent(); sb.Append("{"); firstStack.Push(true); indent++; }
        public void EndObject() { indent--; sb.Append("\n"); for (int i = 0; i < indent; i++) sb.Append("  "); sb.Append("}"); firstStack.Pop(); }
        public void StartArray() { Comma(); Indent(); sb.Append("["); firstStack.Push(true); indent++; }
        public void EndArray() { indent--; sb.Append("\n"); for (int i = 0; i < indent; i++) sb.Append("  "); sb.Append("]"); firstStack.Pop(); }

        public void Key(string name)
        {
            Comma();
            sb.Append("\n");
            for (int i = 0; i < indent; i++) sb.Append("  ");
            EscapeStringInto(name);
            sb.Append(": ");
            firstStack.Push(true);
        }

        public void String(string value) { Comma(); EscapeStringInto(value); MarkWritten(); }
        public void Bool(bool b) { Comma(); sb.Append(b ? "true" : "false"); MarkWritten(); }
        public void Null() { Comma(); sb.Append("null"); MarkWritten(); }
        public void Raw(string r) { Comma(); sb.Append(r); MarkWritten(); }

        private void Comma()
        {
            if (firstStack.Count == 0) return;
            if (firstStack.Peek())
            {
                firstStack.Pop();
                firstStack.Push(false);
                return;
            }
            sb.Append(",");
        }

        private void MarkWritten()
        {
            if (firstStack.Count > 1 && firstStack.Peek())
            {
                firstStack.Pop();
            }
        }

        private void Indent()
        {
            if (sb.Length == 0) return;
            sb.Append("\n");
            for (int i = 0; i < indent; i++) sb.Append("  ");
        }

        private void EscapeStringInto(string value)
        {
            if (value == null) { sb.Append("null"); return; }
            sb.Append('"');
            foreach (char c in value)
            {
                switch (c)
                {
                    case '\\': sb.Append("\\\\"); break;
                    case '"': sb.Append("\\\""); break;
                    case '\n': sb.Append("\\n"); break;
                    case '\r': sb.Append("\\r"); break;
                    case '\t': sb.Append("\\t"); break;
                    default:
                        if (c < 0x20) sb.AppendFormat("\\u{0:X4}", (int)c);
                        else sb.Append(c);
                        break;
                }
            }
            sb.Append('"');
        }

        public override string ToString() => sb.ToString();
    }
}
