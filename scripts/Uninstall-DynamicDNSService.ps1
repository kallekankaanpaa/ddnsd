$prevPwd = $PWD; Set-Location -ErrorAction Stop -LiteralPath $PSScriptRoot
try {
    $service_name = Get-Content -Path ..\service_name.in | Out-String
    Remove-Service -Name $service_name
    sc.exe delete $service_name
}
finally {
    $prevPwd | Set-Location
}