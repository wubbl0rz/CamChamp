using System.Runtime.InteropServices;

namespace backend;

public struct CUint128
{
  public UInt64 upper;
  public UInt64 lower;
}

public struct ConnectionResult() : IDisposable
{
  public UInt32 ClientId;
  private readonly IntPtr _offer = IntPtr.Zero;

  public string? GetOffer()
  {
    var str = Marshal.PtrToStringUTF8(_offer);
    return str;
  }

  public void Dispose()
  {
    // TODO release managed resources here
  }
}

public unsafe class InteropGo
{
  [DllImport("webrtc_linux", EntryPoint = "freeStr")]
  public static extern void freeStr(IntPtr ptr);
  
  [DllImport("webrtc_linux", EntryPoint = "initNetwork")]
  public static extern void initNetwork();
  
  [DllImport("webrtc_linux", EntryPoint = "createTrack")]
  public static extern void createTrack();
  
  [DllImport("webrtc_linux", EntryPoint = "createConnection")]
  public static extern ConnectionResult createConnection();
  
  [DllImport("webrtc_linux", EntryPoint = "setAnswer")]
  public static extern void setAnswer(int id, [MarshalAs(UnmanagedType.LPUTF8Str)]string sdp);
}

public unsafe class InteropRust
{
  [DllImport("webrtcrust", EntryPoint = "init")]
  public static extern void Init();
  [DllImport("webrtcrust", EntryPoint = "create_connection")]
  public static extern ConnectionResult CreateConnection();
  [DllImport("webrtcrust", EntryPoint = "create_track")]
  public static extern ConnectionResult CreateTrack();
  [DllImport("webrtcrust", EntryPoint = "set_answer")]
  public static extern void SetAnswer(UInt64 id, [MarshalAs(UnmanagedType.LPUTF8Str)]string sdp);
}
