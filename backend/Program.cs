using System.Text.Json;
using System.Text.Json.Nodes;
using backend;

var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();
app.UseFileServer();

InteropGo.initNetwork();
InteropGo.createTrack();

app.MapGet("/webrtc/start", () =>
{
  using var result = InteropGo.createConnection();

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
  var id = json["id"]!.GetValue<int>();
  var sdp = json["sdp"]!.GetValue<string>();
 
  Console.WriteLine("ANSWER:");
  Console.WriteLine(sdp);

  InteropGo.setAnswer(id, sdp);
});

app.Run();
