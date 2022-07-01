param([System.String]$env = "debug") 

$prevPwd = $PWD; Set-Location -ErrorAction Stop -LiteralPath $PSScriptRoot
try {
    $service_name = Get-Content -Path ..\service_name.in | Out-String
    New-Service -Name $service_name -BinaryPathName $PWD\..\target\$env\ddnsd.exe
}
finally {
    $prevPwd | Set-Location
}