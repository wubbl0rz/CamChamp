using System.Text.Json;
using System.Text.Json.Nodes;
using backend;

var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();
app.UseFileServer();

InteropRust.Init();
InteropRust.CreateTrack();

// var r = InteropRust.CreateConnection();
//
// var i = new UInt128(r.ClientId.lower, r.ClientId.upper);
//
// Console.WriteLine(i);

app.MapGet("/webrtc/start", () =>
{
  using var result = InteropRust.CreateConnection();

  Console.WriteLine("OFFER: ");
  Console.WriteLine(result.GetOffer());

  return Results.Json(new
  {
    sdp = result.GetOffer(),
    result.ClientId
  });
});

app.MapPost("/webrtc/answer", (JsonObject json) =>
{
  var id = json["id"]!.GetValue<UInt64>();
  var sdp = json["sdp"]!.GetValue<string>();
 
  Console.WriteLine("ANSWER:");
  Console.WriteLine(sdp);

  InteropRust.SetAnswer(id, sdp);
});

app.Run();