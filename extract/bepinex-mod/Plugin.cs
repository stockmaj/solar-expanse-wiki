using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using BepInEx;
using HarmonyLib;
using Manager;
using UnityEngine;

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

        private void Awake()
        {
            Log = Logger;
            var marker = Path.Combine(Application.streamingAssetsPath, MarkerFileName);
            if (File.Exists(marker))
            {
                Log.LogInfo($"Marker present at {marker}; dumper will not run.  Delete it to re-dump.");
                return;
            }
            new Harmony(PluginGuid).PatchAll();
            Log.LogInfo("Harmony patches installed; waiting for AllScriptableObjectManager.InitializeSingleton.");
        }
    }

    [HarmonyPatch(typeof(AllScriptableObjectManager), "InitializeSingleton")]
    internal static class DumpPatch
    {
        private static bool dumped;

        // ReSharper disable once UnusedMember.Local
        private static void Postfix(AllScriptableObjectManager __instance)
        {
            if (dumped) return;
            dumped = true;

            var dir = Application.streamingAssetsPath;
            try
            {
                Plugin.Log.LogInfo("InitializeSingleton postfix fired — running dump.");
                var json = Dumper.Dump(__instance);
                File.WriteAllText(Path.Combine(dir, Plugin.OutputFileName), json);
                File.WriteAllText(Path.Combine(dir, Plugin.MarkerFileName), DateTime.UtcNow.ToString("O"));
                Plugin.Log.LogInfo($"Wrote {json.Length:N0} characters to {Plugin.OutputFileName}");
            }
            catch (Exception ex)
            {
                Plugin.Log.LogError($"Dump failed: {ex}");
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
            writer.EndObject();
            Plugin.Log.LogInfo($"Collected {byType.Sum(p => p.Value.Count):N0} objects across {byType.Count} type(s).");
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
